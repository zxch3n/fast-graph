import { add } from '../src';
import { describe, it, expect } from 'vitest';

describe('add', () => {
  it('adds two numbers', () => {
    expect(add(1, 1)).toBe(2);
  });
});
