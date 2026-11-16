import { toRaw } from 'vue'
import { expect, test } from 'vitest'
import { createProgresses } from '../src/progresses.ts'
import type { FrameScheduler } from '../src/progresses.ts'
import type { ProgressData } from '../src/panes/DownloadPane/progress.ts'

function createFrameHarness() {
  const callbacks: Array<() => void> = []
  const scheduleFrame: FrameScheduler = (callback) => callbacks.push(callback)

  return {
    scheduleFrame,
    get pendingFrames() {
      return callbacks.length
    },
    runFrame() {
      const callback = callbacks.shift()
      expect(callback, 'expected a pending animation frame').toBeDefined()
      if (callback === undefined) {
        throw new Error('expected a pending animation frame')
      }
      callback()
    },
  }
}

function progress(taskId: string, percentage = 0, state: ProgressData['state'] = 'Downloading'): ProgressData {
  return {
    task_id: taskId,
    state,
    percentage,
    stateIndicator: '下载中',
    taskIndicator: '',
  } as ProgressData
}

test('coalesces repeated writes to the same task into one frame', () => {
  const frames = createFrameHarness()
  const store = createProgresses(frames.scheduleFrame)

  store.setProgress(progress('task-1'))
  frames.runFrame()

  for (let percentage = 1; percentage <= 100; percentage += 1) {
    expect(store.patchProgress('task-1', { percentage })).toBe(true)
  }

  expect(frames.pendingFrames).toBe(1)
  expect(store.progresses.value.get('task-1')?.percentage).toBe(0)

  frames.runFrame()

  expect(store.progresses.value.get('task-1')?.percentage).toBe(100)
  expect(frames.pendingFrames).toBe(0)
})

test('does not write to clean reactive records', () => {
  const frames = createFrameHarness()
  const store = createProgresses(frames.scheduleFrame)

  store.setProgress(progress('task-0'))
  for (let index = 1; index < 5_000; index += 1) {
    store.setProgress(progress(`task-${index}`, 100, 'Completed'))
  }
  expect(frames.pendingFrames).toBe(1)
  frames.runFrame()

  expect(store.uncompletedProgresses.value.size).toBe(1)
  expect(store.completedProgresses.value.size).toBe(4_999)

  const clean = store.progresses.value.get('task-4999')
  expect(clean).toBeDefined()
  if (clean === undefined) {
    throw new Error('expected the clean task to exist')
  }

  let cleanIndicator = clean.stateIndicator
  let cleanWrites = 0
  Object.defineProperty(toRaw(clean), 'stateIndicator', {
    configurable: true,
    get: () => cleanIndicator,
    set: (value: string) => {
      cleanWrites += 1
      cleanIndicator = value
    },
  })

  store.patchProgress('task-0', { percentage: 50 })
  frames.runFrame()

  expect(store.progresses.value.get('task-0')?.percentage).toBe(50)
  expect(cleanWrites).toBe(0)
})

test('preserves event order in the canonical state before a frame is rendered', () => {
  const frames = createFrameHarness()
  const store = createProgresses(frames.scheduleFrame)

  store.setProgress(progress('task-1', 10))
  expect(store.patchProgress('task-1', { percentage: 20 })).toBe(true)
  expect(store.deleteProgress('task-1')).toBe(true)
  expect(store.patchProgress('task-1', { percentage: 30 })).toBe(false)
  store.setProgress(progress('task-1', 40))
  expect(store.patchProgress('task-1', { percentage: 50 })).toBe(true)

  expect(frames.pendingFrames).toBe(1)
  frames.runFrame()

  expect(store.progresses.value.get('task-1')?.percentage).toBe(50)
})

test('updates the state indexes when a task changes category or is deleted', () => {
  const frames = createFrameHarness()
  const store = createProgresses(frames.scheduleFrame)

  store.setProgress(progress('task-1'))
  frames.runFrame()
  expect(store.uncompletedProgresses.value.has('task-1')).toBe(true)
  expect(store.completedProgresses.value.has('task-1')).toBe(false)

  store.patchProgress('task-1', { state: 'Completed' })
  frames.runFrame()
  expect(store.uncompletedProgresses.value.has('task-1')).toBe(false)
  expect(store.completedProgresses.value.has('task-1')).toBe(true)

  store.deleteProgress('task-1')
  frames.runFrame()
  expect(store.completedProgresses.value.has('task-1')).toBe(false)
  expect(store.progresses.value.has('task-1')).toBe(false)
})
