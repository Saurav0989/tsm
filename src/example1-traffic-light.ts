/**
 * Example 1: Traffic Light
 * 
 * Demonstrates basic state machine with strict transitions
 */

import { createMachine, defineConfig } from './index';

// Define state configuration - this is the SOURCE OF TRUTH
// TypeScript will enforce these transitions at compile time
const trafficLightConfig = defineConfig({
  red: ['green'] as const,
  yellow: ['red'] as const,
  green: ['yellow'] as const,
});

// Create the machine
const trafficLight = createMachine('traffic-light', trafficLightConfig)
  .setInitial('red')
  .build();

async function testTrafficLight() {
  console.log('\n=== Traffic Light State Machine ===\n');
  console.log('Initial state:', trafficLight.getValue());

  // Valid transition: red -> green ✅
  console.log('\n1. Attempting valid transition: red -> green');
  let result = await trafficLight.transition('green');
  console.log('Success:', result.success);
  console.log('Current state:', trafficLight.getValue());

  // Valid transition: green -> yellow ✅
  console.log('\n2. Attempting valid transition: green -> yellow');
  result = await trafficLight.transition('yellow');
  console.log('Success:', result.success);
  console.log('Current state:', trafficLight.getValue());

  // Valid transition: yellow -> red ✅
  console.log('\n3. Attempting valid transition: yellow -> red');
  result = await trafficLight.transition('red');
  console.log('Success:', result.success);
  console.log('Current state:', trafficLight.getValue());

  // COMPILE-TIME ERROR TEST:
  // Uncommenting this line will cause a TypeScript error:
  // trafficLight.transition('yellow'); // ❌ Type error: can't go red -> yellow
  
  // Runtime validation still works for dynamic values
  console.log('\n4. Testing runtime validation with invalid transition');
  trafficLight.reset();
  const invalidTarget = 'yellow' as any;
  result = await trafficLight.transition(invalidTarget);
  console.log('Success:', result.success);
  console.log('Error:', result.error);
  console.log('Current state (should still be red):', trafficLight.getValue());
}

testTrafficLight().catch(console.error);
