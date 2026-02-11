/**
 * Builder API for creating type-safe state machines
 * 
 * Provides a fluent interface for defining states, transitions, guards, and actions
 */

import { StateMachine } from './machine';
import {
  StateConfig,
  States,
  Context,
  Event,
  Guard,
  Action,
  StateMachineDefinition,
} from './types';

export class StateMachineBuilder<
  TConfig extends StateConfig,
  TContext extends Context = Context
> {
  private id: string;
  private config: TConfig;
  private initial?: States<TConfig>;
  private context: TContext;

  constructor(id: string, config: TConfig) {
    this.id = id;
    this.config = config;
    this.context = {} as TContext;
  }

  /**
   * Set initial state
   */
  public setInitial<TState extends States<TConfig>>(state: TState): this {
    this.initial = state;
    return this;
  }

  /**
   * Set initial context
   */
  public setContext(context: TContext): this {
    this.context = context;
    return this;
  }

  /**
   * Build the state machine
   */
  public build(): StateMachine<TConfig, TContext> {
    if (!this.initial) {
      throw new Error('Initial state must be set before building');
    }

    const definition: StateMachineDefinition<TConfig, TContext> = {
      id: this.id,
      initial: this.initial,
      context: this.context,
      config: this.config,
    };

    return new StateMachine(definition);
  }
}

/**
 * Helper function to create a state machine builder with type inference
 */
export function createMachine<TConfig extends StateConfig, TContext extends Context = {}>(
  id: string,
  config: TConfig
): StateMachineBuilder<TConfig, TContext> {
  return new StateMachineBuilder<TConfig, TContext>(id, config);
}

/**
 * Helper function to create a state machine with context type
 */
export function createMachineWithContext<
  TConfig extends StateConfig,
  TContext extends Context
>(
  id: string,
  config: TConfig,
  context: TContext
): StateMachineBuilder<TConfig, TContext> {
  const builder = new StateMachineBuilder<TConfig, TContext>(id, config);
  builder.setContext(context);
  return builder;
}

/**
 * Type helper to define state config with better inference
 */
export function defineConfig<T extends StateConfig>(config: T): T {
  return config;
}

/**
 * Create a guard function with type safety
 */
export function createGuard<TContext extends Context, TEvent extends Event>(
  fn: Guard<TContext, TEvent>
): Guard<TContext, TEvent> {
  return fn;
}

/**
 * Create an action function with type safety
 */
export function createAction<TContext extends Context, TEvent extends Event>(
  fn: Action<TContext, TEvent>
): Action<TContext, TEvent> {
  return fn;
}

/**
 * Compose multiple actions into one
 */
export function composeActions<TContext extends Context, TEvent extends Event>(
  ...actions: Action<TContext, TEvent>[]
): Action<TContext, TEvent> {
  return async (context, event) => {
    let updatedContext = { ...context };
    for (const action of actions) {
      const result = await action(updatedContext, event);
      if (result && typeof result === 'object') {
        updatedContext = { ...updatedContext, ...result };
      }
    }
    return updatedContext;
  };
}

/**
 * Create a guard that combines multiple guards with AND logic
 */
export function allGuards<TContext extends Context, TEvent extends Event>(
  ...guards: Guard<TContext, TEvent>[]
): Guard<TContext, TEvent> {
  return async (context, event) => {
    for (const guard of guards) {
      const result = await guard(context, event);
      if (!result) {
        return false;
      }
    }
    return true;
  };
}

/**
 * Create a guard that combines multiple guards with OR logic
 */
export function anyGuard<TContext extends Context, TEvent extends Event>(
  ...guards: Guard<TContext, TEvent>[]
): Guard<TContext, TEvent> {
  return async (context, event) => {
    for (const guard of guards) {
      const result = await guard(context, event);
      if (result) {
        return true;
      }
    }
    return false;
  };
}

/**
 * Negate a guard
 */
export function not<TContext extends Context, TEvent extends Event>(
  guard: Guard<TContext, TEvent>
): Guard<TContext, TEvent> {
  return async (context, event) => {
    const result = await guard(context, event);
    return !result;
  };
}
