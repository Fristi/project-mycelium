package co.mycelium.domain

final case class StationUpdate(
    name: Option[String],
    location: Option[String],
    description: Option[String],
    waterSchedule: Option[WateringSchedule]
)
