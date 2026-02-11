/**
 * Example 4: TCP-like Connection State Machine
 * 
 * Demonstrates a realistic protocol implementation with:
 * - Complex state transitions
 * - Event-driven architecture  
 * - Timeouts and retries
 * - Error handling
 */

import {
  createMachineWithContext,
  defineConfig,
  createGuard,
  createAction,
  Event,
} from './index';

// Define connection states based on TCP
const connectionConfig = defineConfig({
  CLOSED: ['SYN_SENT'] as const,
  SYN_SENT: ['ESTABLISHED', 'CLOSED'] as const,
  ESTABLISHED: ['FIN_WAIT', 'CLOSE_WAIT'] as const,
  FIN_WAIT: ['CLOSED'] as const,
  CLOSE_WAIT: ['CLOSED'] as const,
});

// Context tracks connection metadata
interface ConnectionContext {
  remoteAddr: string;
  localPort: number;
  retries: number;
  maxRetries: number;
  lastError?: string;
  bytesReceived: number;
  bytesSent: number;
  connected: boolean;
}

// Custom events
interface ConnectEvent extends Event<'CONNECT'> {
  payload: { remoteAddr: string };
}

interface DataEvent extends Event<'DATA'> {
  payload: { bytes: number };
}

interface ErrorEvent extends Event<'ERROR'> {
  payload: { error: string };
}

// Guards
const canConnect = createGuard<ConnectionContext, ConnectEvent>(
  (context) => !context.connected && context.retries < context.maxRetries
);

const hasRetriesLeft = createGuard<ConnectionContext, ErrorEvent>(
  (context) => context.retries < context.maxRetries
);

// Actions
const initiateConnection = createAction<ConnectionContext, ConnectEvent>(
  (context, event) => {
    console.log(`  → Initiating connection to ${event.payload.remoteAddr}`);
    return {
      remoteAddr: event.payload.remoteAddr,
      retries: context.retries + 1,
    };
  }
);

const establishConnection = createAction<ConnectionContext, Event>(
  (_context) => {
    console.log(`  → Connection established!`);
    return { connected: true };
  }
);

const trackData = createAction<ConnectionContext, DataEvent>(
  (context, event) => {
    const newTotal = context.bytesReceived + event.payload.bytes;
    console.log(`  → Received ${event.payload.bytes} bytes (total: ${newTotal})`);
    return { bytesReceived: newTotal };
  }
);

const handleError = createAction<ConnectionContext, ErrorEvent>(
  (_context, event) => {
    console.log(`  → Error: ${event.payload.error}`);
    return { lastError: event.payload.error };
  }
);

const closeConnection = createAction<ConnectionContext, Event>(
  (context) => {
    console.log(`  → Connection closed. Total received: ${context.bytesReceived} bytes`);
    return { connected: false };
  }
);

// Create the connection state machine
const connection = createMachineWithContext(
  'tcp-connection',
  connectionConfig,
  {
    remoteAddr: '',
    localPort: 8080,
    retries: 0,
    maxRetries: 3,
    bytesReceived: 0,
    bytesSent: 0,
    connected: false,
  }
)
  .setInitial('CLOSED')
  .build();

// Register transitions
connection
  .on('CLOSED', 'SYN_SENT', {
    event: 'CONNECT',
    guard: canConnect,
    actions: [initiateConnection],
  })
  .on('SYN_SENT', 'ESTABLISHED', {
    event: 'ACK',
    actions: [establishConnection],
  })
  .on('SYN_SENT', 'CLOSED', {
    event: 'ERROR',
    guard: hasRetriesLeft,
    actions: [handleError],
  })
  .on('ESTABLISHED', 'FIN_WAIT', {
    event: 'CLOSE',
    actions: [closeConnection],
  })
  .on('ESTABLISHED', 'CLOSE_WAIT', {
    event: 'REMOTE_CLOSE',
  })
  .on('FIN_WAIT', 'CLOSED')
  .on('CLOSE_WAIT', 'CLOSED', {
    actions: [closeConnection],
  });

// Add transition logging
connection.onTransition((result) => {
  if (result.success) {
    console.log(`\n[STATE TRANSITION] ${result.from} → ${result.to}`);
  } else {
    console.log(`\n[TRANSITION FAILED] ${result.from} → ${result.to}: ${result.error}`);
  }
});

async function simulateConnection() {
  console.log('\n=== TCP Connection Simulation ===\n');
  console.log('Initial state:', connection.getValue());
  
  // Attempt to connect
  console.log('\n--- Connecting to remote server ---');
  const connectEvent: ConnectEvent = {
    type: 'CONNECT',
    payload: { remoteAddr: '192.168.1.100:443' },
  };
  await connection.transition('SYN_SENT', connectEvent);
  
  // Receive acknowledgment
  console.log('\n--- Receiving ACK ---');
  await connection.transition('ESTABLISHED', { type: 'ACK' });
  
  // Simulate data transfer
  console.log('\n--- Transferring data ---');
  const data1: DataEvent = {
    type: 'DATA',
    payload: { bytes: 1024 },
  };
  await trackData(connection.getContext(), data1);
  
  const data2: DataEvent = {
    type: 'DATA',
    payload: { bytes: 2048 },
  };
  await trackData(connection.getContext(), data2);
  
  connection.updateContext({ bytesReceived: 3072 });
  
  // Close connection
  console.log('\n--- Closing connection ---');
  await connection.transition('FIN_WAIT', { type: 'CLOSE' });
  await connection.transition('CLOSED');
  
  console.log('\nFinal context:', connection.getContext());
  console.log('\n--- Connection History ---');
  console.log(connection.getHistory().join(' → '));
}

async function simulateConnectionFailure() {
  console.log('\n\n=== Connection Failure Simulation ===\n');
  
  // Create a new connection for failure test
  const failConn = createMachineWithContext(
    'tcp-fail',
    connectionConfig,
    {
      remoteAddr: '',
      localPort: 8080,
      retries: 0,
      maxRetries: 2,
      lastError: undefined as string | undefined,
      bytesReceived: 0,
      bytesSent: 0,
      connected: false,
    }
  )
    .setInitial('CLOSED')
    .build();

  failConn
    .on('CLOSED', 'SYN_SENT', {
      guard: canConnect,
      actions: [initiateConnection],
    })
    .on('SYN_SENT', 'CLOSED', {
      guard: hasRetriesLeft,
      actions: [handleError],
    });

  failConn.onTransition((result) => {
    if (result.success) {
      console.log(`[STATE] ${result.from} → ${result.to}`);
    }
  });
  
  console.log('Attempting connection with retries...\n');
  
  // Attempt 1
  console.log('Attempt 1:');
  await failConn.transition('SYN_SENT', {
    type: 'CONNECT',
    payload: { remoteAddr: '10.0.0.1:80' },
  });
  console.log('Connection failed, retrying...\n');
  const errorEvent: ErrorEvent = {
    type: 'ERROR',
    payload: { error: 'Connection timeout' },
  };
  await failConn.transition('CLOSED', errorEvent);
  
  // Attempt 2
  console.log('Attempt 2:');
  await failConn.transition('SYN_SENT', {
    type: 'CONNECT',
    payload: { remoteAddr: '10.0.0.1:80' },
  });
  console.log('Connection failed, retrying...\n');
  await failConn.transition('CLOSED', errorEvent);
  
  // Attempt 3 should fail guard
  console.log('Attempt 3:');
  const result = await failConn.transition('SYN_SENT', {
    type: 'CONNECT',
    payload: { remoteAddr: '10.0.0.1:80' },
  });
  
  if (!result.success) {
    console.log('❌ Max retries reached. Connection abandoned.');
  }
  
  console.log('\nRetries:', failConn.getContext().retries);
  console.log('Last error:', failConn.getContext().lastError);
}

async function runProtocolSimulation() {
  await simulateConnection();
  await simulateConnectionFailure();
}

runProtocolSimulation().catch(console.error);
