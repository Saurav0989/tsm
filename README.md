# TSM - Type-Safe State Machines for TypeScript

**Catch invalid state transitions at compile time, not runtime.**

## The Problem

Traditional state machine libraries validate transitions at runtime:

```typescript
// Using typical library
machine.transition('red', 'yellow'); // ✅ Compiles fine
// 💥 Runtime error: "Invalid transition red -> yellow"
```

This means:
- Bugs aren't caught until runtime
- No IDE autocomplete for valid transitions
- Refactoring is error-prone
- Testing burden is high

## The Solution

TSM uses TypeScript's type system to enforce state machine correctness **at compile time**:

```typescript
// Using TSM
const config = defineConfig({
  red: ['green'] as const,
  green: ['yellow'] as const,
  yellow: ['red'] as const,
});

const machine = createMachine('traffic', config)
  .setInitial('red')
  .build();

await machine.transition('green');  // ✅ Compiles - valid transition
await machine.transition('yellow'); // ❌ TypeScript error - invalid transition
```

**Invalid transitions are TypeScript compilation errors.** They never make it to production.

## Key Features

### 1. Compile-Time Safety
- Invalid transitions are caught by TypeScript
- IDE autocomplete for valid states
- Refactoring is safe - compiler catches breaking changes

### 2. Runtime Validation
- Still validates at runtime for dynamic data
- Clear error messages
- Type-safe throughout

### 3. Guards (Conditional Transitions)
```typescript
const canProcess = createGuard<Context, Event>(
  (context) => context.balance > 0
);

machine.on('pending', 'processing', { guard: canProcess });
```

### 4. Actions (Side Effects)
```typescript
const logTransition = createAction<Context, Event>(
  (context) => console.log('Transitioning...') 
);

const updateBalance = createAction<Context, Event>(
  (context) => ({ balance: context.balance - 10 })
);

machine.on('processing', 'completed', {
  actions: [logTransition, updateBalance]
});
```

### 5. Context Data
```typescript
interface OrderContext {
  orderId: string;
  total: number;
  attempts: number;
}

const machine = createMachineWithContext(
  'order',
  config,
  { orderId: '123', total: 99.99, attempts: 0 }
);

machine.getContext(); // Fully typed
machine.updateContext({ attempts: 1 }); // Type-safe updates
```

### 6. History Tracking
```typescript
machine.getHistory(); // ['pending', 'processing', 'completed']
```

### 7. Transition Callbacks
```typescript
machine.onTransition((result) => {
  console.log(`Transitioned from ${result.from} to ${result.to}`);
});
```

## Installation

```bash
npm install tsm
```

## Quick Start

### Basic State Machine

```typescript
import { createMachine, defineConfig } from 'tsm';

// Define your states and valid transitions
const config = defineConfig({
  idle: ['loading'] as const,
  loading: ['success', 'error'] as const,
  success: ['idle'] as const,
  error: ['idle'] as const,
});

// Create machine
const machine = createMachine('data-fetcher', config)
  .setInitial('idle')
  .build();

// Use it
await machine.transition('loading');
await machine.transition('success');
console.log(machine.getValue()); // 'success'
```

### With Context and Guards

```typescript
import { 
  createMachineWithContext, 
  defineConfig,
  createGuard,
  createAction 
} from 'tsm';

const config = defineConfig({
  locked: ['unlocked'] as const,
  unlocked: ['locked'] as const,
});

interface DoorContext {
  code: string;
  correctCode: string;
  attempts: number;
}

const isCorrectCode = createGuard<DoorContext, any>(
  (context) => context.code === context.correctCode
);

const incrementAttempts = createAction<DoorContext, any>(
  (context) => ({ attempts: context.attempts + 1 })
);

const machine = createMachineWithContext(
  'door',
  config,
  { code: '', correctCode: '1234', attempts: 0 }
)
  .setInitial('locked')
  .build();

machine.on('locked', 'unlocked', {
  guard: isCorrectCode,
  actions: [incrementAttempts]
});

// Set code
machine.updateContext({ code: '1234' });

// Try to unlock
const result = await machine.transition('unlocked');
console.log(result.success); // true if code was correct
```

## Real-World Examples

### 1. HTTP Request State

```typescript
const requestConfig = defineConfig({
  idle: ['loading'] as const,
  loading: ['success', 'error'] as const,
  success: ['idle'] as const,
  error: ['idle', 'loading'] as const, // can retry
});
```

### 2. User Authentication Flow

```typescript
const authConfig = defineConfig({
  loggedOut: ['loggingIn'] as const,
  loggingIn: ['loggedIn', 'loginFailed'] as const,
  loggedIn: ['loggingOut'] as const,
  loggingOut: ['loggedOut'] as const,
  loginFailed: ['loggedOut', 'loggingIn'] as const,
});
```

### 3. Order Fulfillment

```typescript
const orderConfig = defineConfig({
  pending: ['processing', 'cancelled'] as const,
  processing: ['shipped', 'failed'] as const,
  shipped: ['delivered'] as const,
  delivered: [] as const,
  cancelled: [] as const,
  failed: ['pending'] as const, // retry
});
```

### 4. Game Character States

```typescript
const characterConfig = defineConfig({
  idle: ['walking', 'attacking'] as const,
  walking: ['idle', 'running', 'attacking'] as const,
  running: ['walking', 'idle'] as const,
  attacking: ['idle'] as const,
});
```

## API Reference

### Core Functions

#### `defineConfig<T>(config: T): T`
Helper for defining state configuration with type inference.

#### `createMachine<TConfig>(id: string, config: TConfig): StateMachineBuilder`
Create a state machine builder.

#### `createMachineWithContext<TConfig, TContext>(id, config, context): StateMachineBuilder`
Create a state machine with initial context.

### StateMachine Methods

#### `transition<TTo>(to: TTo, event?: Event): Promise<TransitionResult>`
Transition to a new state. Returns result indicating success/failure.

#### `getValue(): States<TConfig>`
Get current state.

#### `getContext(): TContext`
Get current context.

#### `updateContext(updates: Partial<TContext>): void`
Update context without changing state.

#### `getValidTransitions(): ValidTargets[]`
Get all valid transitions from current state.

#### `getHistory(): States[]`
Get complete transition history.

#### `reset(): void`
Reset to initial state.

#### `on<TFrom, TTo>(from, to, options?): this`
Register transition with optional guard and actions.

#### `onTransition(callback): () => void`
Register callback for all transitions. Returns unsubscribe function.

### Helper Functions

#### `createGuard<TContext, TEvent>(fn): Guard`
Create a type-safe guard function.

#### `createAction<TContext, TEvent>(fn): Action`
Create a type-safe action function.

#### `composeActions(...actions): Action`
Combine multiple actions into one.

#### `allGuards(...guards): Guard`
Combine guards with AND logic.

#### `anyGuard(...guards): Guard`
Combine guards with OR logic.

#### `not(guard): Guard`
Negate a guard.

## TypeScript Configuration

TSM requires TypeScript 4.1+ for template literal types.

Recommended `tsconfig.json`:
```json
{
  "compilerOptions": {
    "strict": true,
    "target": "ES2020",
    "module": "commonjs"
  }
}
```

## Comparison to Other Libraries

### XState
- **XState**: Runtime validation, complex API, heavyweight
- **TSM**: Compile-time validation, simple API, lightweight

### robot3
- **robot3**: Functional but basic type safety
- **TSM**: Full compile-time guarantees on transitions

### State.js
- **State.js**: JavaScript-first, minimal typing
- **TSM**: TypeScript-first, maximum type safety

## When to Use TSM

**Use TSM when:**
- You want compile-time guarantees
- State transitions are known at build time
- You value type safety and IDE support
- You need guards, actions, and context

**Don't use TSM when:**
- State transitions are completely dynamic
- You need visualization tools (use XState)
- You need hierarchical/parallel states
- You need interpreter/SCXML compliance

## Performance

TSM has zero runtime overhead for type checking (it's all compile-time).

Runtime performance:
- State transitions: O(1)
- Guard evaluation: O(number of guards)
- Action execution: O(number of actions)

Memory: ~1KB per machine instance (context size dependent)

## Design Decisions

### Why Template Literal Types?
Enables compile-time validation of transitions without code generation.

### Why No Hierarchical States?
Keeps implementation simple and compile-time overhead low. For complex hierarchies, use XState.

### Why Async Actions?
Supports real-world use cases (API calls, database operations) without blocking.

### Why Immutable State?
Prevents accidental mutations and makes debugging easier.

## TSM Ecosystem

TSM is part of a larger ecosystem of research projects focused on formal verification and autonomous discovery:

### 🛡️ [Lattice](file:///Users/sauravkumar/Movies/lattice/)
**Verifiable Distributed State Machine**
- Deterministic Simulation Testing (DST)
- Real-time Formal Verification via Shadow Models
- Post-Quantum Cryptography (Kyber/Dilithium)

### 🧬 [Universal Discovery Engine (UDE)](file:///Users/sauravkumar/Movies/TSM%20-%20Type-Safe%20State%20Machines%20for%20TypeScript/ude/)
**Autonomous Mathematical Discovery System**
- Systematic exploration of 10^42+ theorem spaces
- Formal verification via Lean 4 & Z3
- Neural pattern learning for guided search

---

## Contributing
// ... (rest of the content)
