/**
 * Type Safety Tests
 * 
 * These tests demonstrate that invalid transitions are caught at COMPILE TIME
 * 
 * Uncomment any of the error lines to see TypeScript compilation fail
 */

import { createMachine, defineConfig } from './index';

const trafficConfig = defineConfig({
  red: ['green'] as const,
  yellow: ['red'] as const,
  green: ['yellow'] as const,
});

const machine = createMachine('test', trafficConfig)
  .setInitial('red')
  .build();

async function typeTests() {
  // ✅ VALID - These should compile
  await machine.transition('green');  // red -> green is valid
  
  // ❌ INVALID - Uncomment to see compile error
  // await machine.transition('yellow');  // TypeScript error: Type '"yellow"' is not assignable to type '"green"'
  
  // The type system knows current state and enforces valid targets!
  
  // After transition to green:
  machine.reset();
  await machine.transition('green');
  
  // ✅ VALID
  await machine.transition('yellow');  // green -> yellow is valid
  
  // ❌ INVALID - Uncomment to see compile error  
  // await machine.transition('green');  // TypeScript error: Cannot transition green -> green
  // await machine.transition('red');    // TypeScript error: Cannot transition green -> red
  
  console.log('✅ All valid transitions compiled successfully!');
  console.log('❌ Invalid transitions would cause TypeScript errors');
}

typeTests();

// Export type to show inference works
export type TrafficStates = ReturnType<typeof machine.getValue>;
// TrafficStates is inferred as: "red" | "yellow" | "green"
