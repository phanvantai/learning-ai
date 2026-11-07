import { describe, expect, it } from 'vitest';
import { soPhutDoc } from '../src/bai-chia-se';

const chu = (n: number) => Array.from({ length: n }, () => 'chữ').join(' ');

describe('soPhutDoc', () => {
  it('bài ngắn vẫn tính 1 phút', () => {
    expect(soPhutDoc('')).toBe(1);
    expect(soPhutDoc(chu(10))).toBe(1);
  });

  it('làm tròn lên theo tốc độ 220 chữ mỗi phút', () => {
    expect(soPhutDoc(chu(220))).toBe(1);
    expect(soPhutDoc(chu(221))).toBe(2);
    expect(soPhutDoc(chu(1100))).toBe(5);
  });

  it('không đếm khoảng trắng và xuống dòng thừa', () => {
    expect(soPhutDoc(`  ${chu(220)}\n\n\n  `)).toBe(1);
  });
});
