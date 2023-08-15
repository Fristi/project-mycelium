package co.mycelium.domain

import cron4s.CronExpr

import scala.concurrent.duration.FiniteDuration

sealed trait WateringSchedule {
  val period: FiniteDuration
}

object WateringSchedule {
  case class Interval(schedule: CronExpr, period: FiniteDuration) extends WateringSchedule

  case class Threshold(belowSoilPf: Int, period: FiniteDuration) extends WateringSchedule
}
