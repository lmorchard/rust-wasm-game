import { Main } from "../pkg";
import { memory } from "../pkg/wasm_game_bg";

const TARGET_FPS = 60;
const TARGET_DURATION = 1000 / TARGET_FPS;
const PI2 = Math.PI * 2;

const canvas = document.getElementById("main");
canvas.height = 700;
canvas.width = 700;

const ctx = canvas.getContext("2d");

function init() {
  const main = Main.new();
  const driver = new Driver({ main });
  driver.start();
}

class Driver {
  constructor(options = {
    main: null,
    debug: true,
  }) {
    this.main = options.main;
    this.isRunning = false;
    this.isPaused = false;
    this.debug = options.debug;
    this.updateDuration = TARGET_DURATION;
    this.maxUpdateDelta = TARGET_DURATION * 5;
    this.updateAccumulator = 0;
    this.lastUpdateTime = 0;
    this.lastDrawTime = 0;
    this.updateLoop = this.updateLoop.bind(this);
    this.drawLoop = this.drawLoop.bind(this);

  }

  start() {
    if (this.isRunning) { return; }
    this.isRunning = true;
    this.lastUpdateTime = Date.now();
    this.lastDrawTime = 0;
    setTimeout(this.updateLoop, this.updateDuration);
    requestAnimationFrame(this.drawLoop);
    return this;
  }

  stop() {
    this.isRunning = false;
    this.main.stop();
    return this;
  }

  pause() {
    this.isPaused = true;
    this.main.pause();
    return this;
  }

  resume() {
    this.isPaused = false;
    this.main.resume();
    return this;
  }

  updateLoop() {
    const timeNow = Date.now();
    const timeDelta = Math.min(timeNow - this.lastUpdateTime, this.maxUpdateDelta);
    this.lastUpdateTime = timeNow;
    if (!this.isPaused) {
      this.updateAccumulator += timeDelta;
      while (this.updateAccumulator >= this.updateDuration) {
        this.main.update(this.updateDuration);
        this.updateAccumulator -= this.updateDuration;
      }
    }
    if (this.isRunning) {
      setTimeout(this.updateLoop, this.updateDuration);
    }
  }

  drawLoop(timestamp) {
    if (!this.lastDrawTime) { this.lastDrawTime = timestamp; }
    const timeDelta = timestamp - this.lastDrawTime;
    this.lastDrawTime = timestamp;
    if (!this.isPaused) {
      this.draw(timeDelta);
    }
    if (this.isRunning) {
      requestAnimationFrame(this.drawLoop);
    }
  }

  draw(timeDelta) {
    const size = this.main.get_render_size();
    const pos_x = new Float32Array(memory.buffer, this.main.get_render_pos_x(), size);
    const pos_y = new Float32Array(memory.buffer, this.main.get_render_pos_y(), size);
    const orientation = new Float32Array(memory.buffer, this.main.get_render_orientation(), size);
    const asset_ids = new Uint8Array(memory.buffer, this.main.get_render_asset_ids(), size);

    document.getElementById("result").innerText = JSON.stringify({
      size, pos_x, pos_y, orientation, asset_ids
    }, null, " ");

    ctx.fillStyle = 'rgba(0, 0, 0, 1.0)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const scale = 0.25;
    for (let idx = 0; idx < size; idx++) {
      ctx.save();
      ctx.translate(pos_x[idx], pos_y[idx]);
      ctx.rotate(orientation[idx]);
      ctx.scale(scale, scale);
      ctx.lineWidth = 1.0 / scale;
      ctx.strokeStyle = "rgba(255, 255, 255, 0.9)";
      ctx.beginPath();
      ctx.arc(0, 0, 50, 0, PI2, true);
      ctx.moveTo(0, 0);
      ctx.lineTo(0, -50);
      ctx.moveTo(0, 0);
      ctx.stroke();
      ctx.restore();
    }
  }
}

init();
