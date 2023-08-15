package co.mycelium.domain

import java.time.Instant
import java.util.UUID

final case class StationInsert(
    mac: String,
    name: String,
    location: String,
    description: String,
    wateringSchedule: WateringSchedule
) {
  def toStation(id: UUID, created: Instant, userId: String): Station =
    Station(
      id = id,
      mac = mac,
      name = name,
      location = location,
      description = description,
      wateringSchedule = wateringSchedule,
      userId = userId,
      created = created,
      updated = None
    )
}
