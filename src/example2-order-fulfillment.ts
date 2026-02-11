/**
 * Example 2: Order Fulfillment
 * 
 * Demonstrates:
 * - Context data
 * - Guards (conditional transitions)
 * - Actions (side effects)
 * - Events with payloads
 */

import {
  createMachineWithContext,
  defineConfig,
  createGuard,
  createAction,
} from './index';

// Define the order fulfillment workflow
const orderConfig = defineConfig({
  pending: ['processing', 'cancelled'] as const,
  processing: ['shipped', 'failed'] as const,
  shipped: ['delivered'] as const,
  delivered: [] as const,  // Terminal state
  cancelled: [] as const,  // Terminal state
  failed: ['pending'] as const,  // Can retry
});

// Define context type
interface OrderContext {
  orderId: string;
  items: string[];
  total: number;
  attempts: number;
  maxAttempts: number;
}

// Create guards
const canProcess = createGuard<OrderContext, any>(
  (context) => context.total > 0 && context.items.length > 0
);

const canRetry = createGuard<OrderContext, any>(
  (context) => context.attempts < context.maxAttempts
);

// Create actions
const incrementAttempts = createAction<OrderContext, any>(
  (context) => {
    console.log(`  → Incrementing attempts: ${context.attempts} -> ${context.attempts + 1}`);
    return { attempts: context.attempts + 1 };
  }
);

const logShipment = createAction<OrderContext, any>(
  (context) => {
    console.log(`  → Order ${context.orderId} shipped!`);
  }
);

const logDelivery = createAction<OrderContext, any>(
  (context) => {
    console.log(`  → Order ${context.orderId} delivered! Total: $${context.total}`);
  }
);

// Create machine with context
const orderMachine = createMachineWithContext(
  'order-fulfillment',
  orderConfig,
  {
    orderId: 'ORD-12345',
    items: ['Widget', 'Gadget'],
    total: 99.99,
    attempts: 0,
    maxAttempts: 3,
  }
)
  .setInitial('pending')
  .build();

// Register transitions with guards and actions
orderMachine
  .on('pending', 'processing', {
    guard: canProcess,
    actions: [incrementAttempts],
  })
  .on('pending', 'cancelled')
  .on('processing', 'shipped', {
    actions: [logShipment],
  })
  .on('processing', 'failed')
  .on('shipped', 'delivered', {
    actions: [logDelivery],
  })
  .on('failed', 'pending', {
    guard: canRetry,
  });

async function testOrderFulfillment() {
  console.log('\n=== Order Fulfillment State Machine ===\n');
  
  console.log('Initial state:', orderMachine.getValue());
  console.log('Context:', orderMachine.getContext());

  // Happy path
  console.log('\n1. Processing order...');
  await orderMachine.transition('processing');
  console.log('State:', orderMachine.getValue());
  
  console.log('\n2. Shipping order...');
  await orderMachine.transition('shipped');
  console.log('State:', orderMachine.getValue());
  
  console.log('\n3. Delivering order...');
  await orderMachine.transition('delivered');
  console.log('State:', orderMachine.getValue());
  console.log('Final context:', orderMachine.getContext());

  console.log('\n--- Workflow Complete ---');
  console.log('History:', orderMachine.getHistory());
}

async function testFailureAndRetry() {
  console.log('\n=== Testing Failure and Retry ===\n');
  
  // Create a new machine for retry test
  const retryMachine = createMachineWithContext(
    'order-retry-test',
    orderConfig,
    {
      orderId: 'ORD-99999',
      items: ['Test Item'],
      total: 50.00,
      attempts: 0,
      maxAttempts: 2,
    }
  )
    .setInitial('pending')
    .build();

  retryMachine
    .on('pending', 'processing', {
      guard: canProcess,
      actions: [incrementAttempts],
    })
    .on('processing', 'failed')
    .on('failed', 'pending', {
      guard: canRetry,
    });

  console.log('Initial state:', retryMachine.getValue());
  
  console.log('\n1. Attempt 1 - Processing then failing...');
  await retryMachine.transition('processing');
  await retryMachine.transition('failed');
  console.log('State:', retryMachine.getValue());
  console.log('Attempts:', retryMachine.getContext().attempts);

  console.log('\n2. Attempt 2 - Retrying...');
  await retryMachine.transition('pending');
  await retryMachine.transition('processing');
  await retryMachine.transition('failed');
  console.log('State:', retryMachine.getValue());
  console.log('Attempts:', retryMachine.getContext().attempts);

  console.log('\n3. Attempt 3 - Should fail guard (max attempts reached)...');
  const result = await retryMachine.transition('pending');
  console.log('Transition success:', result.success);
  console.log('Error:', result.error);
  console.log('Still in failed state:', retryMachine.getValue());
}

async function runExamples() {
  await testOrderFulfillment();
  await testFailureAndRetry();
}

runExamples().catch(console.error);
