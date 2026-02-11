import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createMachine, defineConfig } from './builder';
import { StateMachine } from './machine';

describe('StateMachine', () => {
    const config = defineConfig({
        idle: ['running', 'error'] as const,
        running: ['idle', 'error'] as const,
        error: ['idle'] as const,
    });

    let machine: StateMachine<typeof config>;

    beforeEach(() => {
        machine = createMachine('test-machine', config)
            .setInitial('idle')
            .build();
    });

    it('should initialize with the initial state', () => {
        expect(machine.getValue()).toBe('idle');
        expect(machine.getHistory()).toEqual(['idle']);
    });

    it('should transition to a valid target state', async () => {
        const result = await machine.transition('running');
        expect(result.success).toBe(true);
        expect(machine.getValue()).toBe('running');
        expect(machine.getHistory()).toEqual(['idle', 'running']);
    });

    it('should fail to transition to an invalid target state', async () => {
        // @ts-expect-error - testing runtime validation for invalid transition
        const result = await machine.transition('invalid');
        expect(result.success).toBe(false);
        expect(machine.getValue()).toBe('idle');
        expect(result.error).toContain('Invalid transition');
    });

    it('should execute guards and allow transition if guard returns true', async () => {
        const guard = vi.fn().mockReturnValue(true);
        machine.on('idle', 'running', { guard });

        const result = await machine.transition('running');
        expect(guard).toHaveBeenCalled();
        expect(result.success).toBe(true);
        expect(machine.getValue()).toBe('running');
    });

    it('should execute guards and block transition if guard returns false', async () => {
        const guard = vi.fn().mockReturnValue(false);
        machine.on('idle', 'running', { guard });

        const result = await machine.transition('running');
        expect(guard).toHaveBeenCalled();
        expect(result.success).toBe(false);
        expect(machine.getValue()).toBe('idle');
        expect(result.error).toBe('All transition guards rejected');
    });

    it('should execute actions during transition', async () => {
        const action = vi.fn();
        machine.on('idle', 'running', { actions: [action] });

        await machine.transition('running');
        expect(action).toHaveBeenCalled();
    });

    it('should update context via actions', async () => {
        interface TestContext { count: number }
        const machineWithContext = createMachine<typeof config, TestContext>('context-machine', config)
            .setInitial('idle')
            .setContext({ count: 0 })
            .build();

        const increment = (context: any) => ({ count: context.count + 1 });
        machineWithContext.on('idle', 'running', { actions: [increment] });

        await machineWithContext.transition('running');
        expect(machineWithContext.getContext().count).toBe(1);
    });

    it('should notify subscribers on transition', async () => {
        const callback = vi.fn();
        machine.onTransition(callback);

        await machine.transition('running');
        expect(callback).toHaveBeenCalledWith(expect.objectContaining({
            success: true,
            from: 'idle',
            to: 'running'
        }));
    });

    it('should reset to initial state', async () => {
        await machine.transition('running');
        machine.reset();
        expect(machine.getValue()).toBe('idle');
        expect(machine.getHistory()).toEqual(['idle']);
    });
});
