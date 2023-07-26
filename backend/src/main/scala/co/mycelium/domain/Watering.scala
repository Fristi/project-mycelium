package co.mycelium.domain

import scala.concurrent.duration.FiniteDuration

final case class Watering(watering: Option[FiniteDuration])
