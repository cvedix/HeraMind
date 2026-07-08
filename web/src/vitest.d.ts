/// <reference types="vitest/globals" />

declare global {
  namespace Vi {
    interface FetchMock {
      mockClear(): void
    }
  }

  var fetch: any
}
