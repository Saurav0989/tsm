/**
 * Core types for type-safe state machines
 * 
 * These types enforce state machine correctness at compile time:
 * - Invalid transitions are TypeScript errors
 * - State context is properly typed
 * - Transition events are validated
 */

/**
 * Extract valid target states for a given source state
 */
export type ValidTargets<
  TConfig extends StateConfig,
  TState extends keyof TConfig
> = TConfig[TState] extends readonly (infer Target)[] ? Target : never;

/**
 * Check if a transition is valid
 */
export type IsValidTransition<
  TConfig extends StateConfig,
  TFrom extends keyof TConfig,
  TTo extends string
> = TTo extends ValidTargets<TConfig, TFrom> ? true : false;

/**
 * State machine configuration type
 * Maps each state to an array of valid target states
 */
export type StateConfig = {
  readonly [state: string]: readonly string[];
};

/**
 * Extract all states from config
 */
export type States<TConfig extends StateConfig> = keyof TConfig;

/**
 * Context data associated with the state machine
 */
export type Context = Record<string, any>;

/**
 * Event that can trigger a transition
 */
export interface Event<TType extends string = string, TPayload = any> {
  readonly type: TType;
  readonly payload?: TPayload;
}

/**
 * Guard function - returns true if transition should be allowed
 */
export type Guard<TContext extends Context, TEvent extends Event> = (
  context: TContext,
  event: TEvent
) => boolean | Promise<boolean>;

/**
 * Action function - side effect executed during transition
 */
export type Action<TContext extends Context, TEvent extends Event> = (
  context: TContext,
  event: TEvent
) => void | Promise<void> | Partial<TContext> | Promise<Partial<TContext>>;

/**
 * Transition definition
 */
export interface Transition<
  TConfig extends StateConfig,
  TContext extends Context,
  TFrom extends States<TConfig>,
  TTo extends ValidTargets<TConfig, TFrom>,
  TEvent extends Event = Event
> {
  readonly from: TFrom;
  readonly to: TTo;
  readonly event?: TEvent['type'];
  readonly guard?: Guard<TContext, TEvent>;
  readonly actions?: Action<TContext, TEvent>[];
}

/**
 * State machine definition
 */
export interface StateMachineDefinition<
  TConfig extends StateConfig,
  TContext extends Context
> {
  readonly id: string;
  readonly initial: States<TConfig>;
  readonly context?: TContext;
  readonly config: TConfig;
}

/**
 * Runtime state of the machine
 */
export interface MachineState<
  TConfig extends StateConfig,
  TContext extends Context
> {
  readonly value: States<TConfig>;
  readonly context: TContext;
  readonly history: States<TConfig>[];
  readonly timestamp: number;
}

/**
 * Transition result
 */
export interface TransitionResult<
  TConfig extends StateConfig,
  TContext extends Context
> {
  readonly success: boolean;
  readonly from: States<TConfig>;
  readonly to: States<TConfig>;
  readonly context: TContext;
  readonly error?: string;
}
