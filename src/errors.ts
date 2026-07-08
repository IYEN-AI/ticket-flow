export class TicketNotFoundError extends Error {
  readonly name = "TicketNotFoundError"

  constructor(readonly ticketId: string) {
    super(`ticket ${ticketId} not found`)
  }
}

export class InvalidTransitionError extends Error {
  readonly name = "InvalidTransitionError"

  constructor(
    readonly from: string,
    readonly to: string,
  ) {
    super(`cannot transition ${from} -> ${to}`)
  }
}

export class ReviewArtifactRequiredError extends Error {
  readonly name = "ReviewArtifactRequiredError"

  constructor(readonly ticketId: string) {
    super(`review requires --artifact for ${ticketId}`)
  }
}

export class InvalidSourceError extends Error {
  readonly name = "InvalidSourceError"

  constructor(readonly source: string) {
    super(`invalid source ${source}`)
  }
}

export class DuplicateDestinationTicketError extends Error {
  readonly name = "DuplicateDestinationTicketError"

  constructor(readonly ticketId: string) {
    super(`duplicate destination ticket ${ticketId}`)
  }
}

export class DuplicateSourceTicketError extends Error {
  readonly name = "DuplicateSourceTicketError"

  constructor(readonly ticketId: string) {
    super(`duplicate source ticket ${ticketId}`)
  }
}
