import '@testing-library/jest-dom/vitest'
import { beforeEach } from 'vitest'
import { afterEach } from 'vitest'

// jsdom has no matchMedia; the Reduced-effects hook needs it.
beforeEach(() => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      addListener: () => undefined,
      removeListener: () => undefined,
      dispatchEvent: () => false,
    }),
  })
})

afterEach(() => {
  document.body.innerHTML = ''
})
