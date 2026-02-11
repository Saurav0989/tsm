import { describe, it, expect, vi } from 'vitest';
import {
    createGuard,
    createAction,
    composeActions,
    allGuards,
    anyGuard,
    not
} from './builder';

describe('Builder API Utilities', () => {
    const context = { value: 10 };
    const event = { type: 'TEST' };

    describe('createGuard', () => {
        it('should return the passed function', () => {
            const fn = () => true;
            expect(createGuard(fn)).toBe(fn);
        });
    });

    describe('createAction', () => {
        it('should return the passed function', () => {
            const fn = () => { };
            expect(createAction(fn)).toBe(fn);
        });
    });

    describe('composeActions', () => {
        it('should execute multiple actions in order and merge context updates', async () => {
            const action1 = vi.fn().mockReturnValue({ a: 1 });
            const action2 = vi.fn().mockReturnValue({ b: 2 });
            const composed = composeActions(action1, action2);

            const result = await composed({}, event);
            expect(action1).toHaveBeenCalled();
            expect(action2).toHaveBeenCalledWith({ a: 1 }, event);
            expect(result).toEqual({ a: 1, b: 2 });
        });
    });

    describe('allGuards', () => {
        it('should return true if all guards pass', async () => {
            const g1 = vi.fn().mockResolvedValue(true);
            const g2 = vi.fn().mockResolvedValue(true);
            const combined = allGuards(g1, g2);

            expect(await combined(context, event)).toBe(true);
        });

        it('should return false if any guard fails', async () => {
            const g1 = vi.fn().mockResolvedValue(true);
            const g2 = vi.fn().mockResolvedValue(false);
            const combined = allGuards(g1, g2);

            expect(await combined(context, event)).toBe(false);
        });
    });

    describe('anyGuard', () => {
        it('should return true if any guard passes', async () => {
            const g1 = vi.fn().mockResolvedValue(false);
            const g2 = vi.fn().mockResolvedValue(true);
            const combined = anyGuard(g1, g2);

            expect(await combined(context, event)).toBe(true);
        });

        it('should return false if all guards fail', async () => {
            const g1 = vi.fn().mockResolvedValue(false);
            const g2 = vi.fn().mockResolvedValue(false);
            const combined = anyGuard(g1, g2);

            expect(await combined(context, event)).toBe(false);
        });
    });

    describe('not', () => {
        it('should negate the guard result', async () => {
            const trueGuard = () => true;
            const falseGuard = () => false;

            expect(await not(trueGuard)(context, event)).toBe(false);
            expect(await not(falseGuard)(context, event)).toBe(true);
        });
    });
});
