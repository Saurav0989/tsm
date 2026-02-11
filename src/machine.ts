/**
 * StateMachine - Runtime engine for type-safe state machines
 * 
 * Enforces valid transitions at both compile time and runtime
 */

import {
  StateConfig,
  States,
  ValidTargets,
  Context,
  Event,
  Guard,
  Action,
  Transition,
  StateMachineDefinition,
  MachineState,
  TransitionResult,
} from './types';

export class StateMachine<
  TConfig extends StateConfig,
  TContext extends Context = Context
> {
  private readonly definition: StateMachineDefinition<TConfig, TContext>;
  private state: MachineState<TConfig, TContext>;
  private transitions: Map<string, Transition<TConfig, TContext, any, any>[]>;
  private onTransitionCallbacks: Array<
    (result: TransitionResult<TConfig, TContext>) => void
  >;

  constructor(definition: StateMachineDefinition<TConfig, TContext>) {
    this.definition = definition;
    this.transitions = new Map();
    this.onTransitionCallbacks = [];

    // Initialize state
    this.state = {
      value: definition.initial,
      context: (definition.context || {}) as TContext,
      history: [definition.initial],
      timestamp: Date.now(),
    };
  }

  /**
   * Get current state
   */
  public getState(): MachineState<TConfig, TContext> {
    return { ...this.state };
  }

  /**
   * Get current state value
   */
  public getValue(): States<TConfig> {
    return this.state.value;
  }

  /**
   * Get current context
   */
  public getContext(): TContext {
    return { ...this.state.context };
  }

  /**
   * Check if transition is valid (runtime check)
   */
  public canTransition<TFrom extends States<TConfig>>(
    from: TFrom,
    to: string
  ): boolean {
    const config = this.definition.config as any;
    const validTargets = config[from as string];
    return validTargets && validTargets.includes(to);
  }

  /**
   * Register a transition with optional guard and actions
   */
  public on<
    TFrom extends States<TConfig>,
    TTo extends ValidTargets<TConfig, TFrom>,
    TEvent extends Event = Event
  >(
    from: TFrom,
    to: TTo,
    options?: {
      event?: TEvent['type'];
      guard?: Guard<TContext, TEvent>;
      actions?: Action<TContext, TEvent>[];
    }
  ): this {
    const key = `${String(from)}->${String(to)}`;
    const transition: Transition<TConfig, TContext, TFrom, TTo, TEvent> = {
      from,
      to,
      event: options?.event,
      guard: options?.guard,
      actions: options?.actions,
    };

    const existing = this.transitions.get(key) || [];
    this.transitions.set(key, [...existing, transition as any]);

    return this;
  }

  /**
   * Transition to a new state
   */
  public async transition<
    TFrom extends States<TConfig>,
    TTo extends ValidTargets<TConfig, TFrom>
  >(
    to: TTo,
    event?: Event
  ): Promise<TransitionResult<TConfig, TContext>> {
    const from = this.state.value as TFrom;

    // Compile-time check is enforced by types
    // Runtime validation
    if (!this.canTransition(from, to as string)) {
      const error = `Invalid transition: ${String(from)} -> ${String(
        to as string
      )}. Valid targets: ${this.definition.config[from].join(', ')}`;
      
      const result: TransitionResult<TConfig, TContext> = {
        success: false,
        from,
        to: from, // Stay in current state
        context: this.state.context,
        error,
      };

      this.notifyTransition(result);
      return result;
    }

    // Find matching transitions
    const key = `${String(from)}->${String(to)}`;
    const transitions = this.transitions.get(key) || [];

    // Filter by event type if provided
    const matchingTransitions = event
      ? transitions.filter(
          (t) => !t.event || t.event === event.type
        )
      : transitions;

    // Check guards
    for (const transition of matchingTransitions) {
      if (transition.guard) {
        const guardResult = await transition.guard(this.state.context, event || { type: 'NONE' });
        if (!guardResult) {
          continue; // Guard rejected, try next transition
        }
      }

      // Execute actions
      let updatedContext = { ...this.state.context };
      if (transition.actions) {
        for (const action of transition.actions) {
          const actionResult = await action(updatedContext, event || { type: 'NONE' });
          if (actionResult && typeof actionResult === 'object') {
            updatedContext = { ...updatedContext, ...actionResult };
          }
        }
      }

      // Update state
      this.state = {
        value: to as States<TConfig>,
        context: updatedContext,
        history: [...this.state.history, to as States<TConfig>],
        timestamp: Date.now(),
      };

      const result: TransitionResult<TConfig, TContext> = {
        success: true,
        from,
        to: to as States<TConfig>,
        context: updatedContext,
      };

      this.notifyTransition(result);
      return result;
    }

    // If we get here, either no transitions matched or all guards rejected
    if (matchingTransitions.length === 0) {
      // Transition is valid but no handlers registered - allow it
      this.state = {
        value: to as States<TConfig>,
        context: this.state.context,
        history: [...this.state.history, to as States<TConfig>],
        timestamp: Date.now(),
      };

      const result: TransitionResult<TConfig, TContext> = {
        success: true,
        from,
        to: to as States<TConfig>,
        context: this.state.context,
      };

      this.notifyTransition(result);
      return result;
    }

    // All guards rejected
    const error = 'All transition guards rejected';
    const result: TransitionResult<TConfig, TContext> = {
      success: false,
      from,
      to: from,
      context: this.state.context,
      error,
    };

    this.notifyTransition(result);
    return result;
  }

  /**
   * Update context without changing state
   */
  public updateContext(updates: Partial<TContext>): void {
    this.state = {
      ...this.state,
      context: { ...this.state.context, ...updates },
    };
  }

  /**
   * Register callback for transitions
   */
  public onTransition(
    callback: (result: TransitionResult<TConfig, TContext>) => void
  ): () => void {
    this.onTransitionCallbacks.push(callback);
    return () => {
      const index = this.onTransitionCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTransitionCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Get transition history
   */
  public getHistory(): States<TConfig>[] {
    return [...this.state.history];
  }

  /**
   * Reset to initial state
   */
  public reset(): void {
    this.state = {
      value: this.definition.initial,
      context: (this.definition.context || {}) as TContext,
      history: [this.definition.initial],
      timestamp: Date.now(),
    };
  }

  /**
   * Get all valid transitions from current state
   */
  public getValidTransitions(): ValidTargets<TConfig, States<TConfig>>[] {
    const current = this.state.value;
    return [...this.definition.config[current]] as any;
  }

  private notifyTransition(result: TransitionResult<TConfig, TContext>): void {
    for (const callback of this.onTransitionCallbacks) {
      try {
        callback(result);
      } catch (error) {
        console.error('Error in transition callback:', error);
      }
    }
  }
}
