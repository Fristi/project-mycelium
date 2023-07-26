package co.mycelium.domain

import java.time.Instant
import java.util.UUID

final case class Station(
                          id: UUID,
                          mac: Array[Byte],
                          name: String,
                          location: String,
                          description: String,
                          wateringSchedule: WateringSchedule,
                          userId: String,
                          created: Instant,
                          updated: Option[Instant]
                        )
