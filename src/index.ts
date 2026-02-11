/**
 * TypeScript State Machine (TSM)
 * 
 * Type-safe state machines with compile-time transition validation
 */

export { StateMachine } from './machine';
export {
  StateMachineBuilder,
  createMachine,
  createMachineWithContext,
  defineConfig,
  createGuard,
  createAction,
  composeActions,
  allGuards,
  anyGuard,
  not,
} from './builder';

export type {
  StateConfig,
  States,
  ValidTargets,
  IsValidTransition,
  Context,
  Event,
  Guard,
  Action,
  Transition,
  StateMachineDefinition,
  MachineState,
  TransitionResult,
} from './types';
