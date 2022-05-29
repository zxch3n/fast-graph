import { Disposable } from './dispose';

export class CanvasControl extends Disposable {
  constructor(private canvas: HTMLCanvasElement) {
    super();
  }
}
