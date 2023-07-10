package co.mycelium.domain

import java.time.Instant
import java.util.UUID

final case class StationLog(stationId: UUID, on: Instant, event: StationEvent)
