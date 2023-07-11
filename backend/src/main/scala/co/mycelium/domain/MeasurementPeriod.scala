package co.mycelium.domain

sealed abstract class MeasurementPeriod(val repr: String)

object MeasurementPeriod {
  case object LastTwentyFourHours extends MeasurementPeriod("last-24-hours")
  case object LastSevenDays extends MeasurementPeriod("last-7-days")
  case object LastTwoWeeks extends MeasurementPeriod("last-2-weeks")
  case object LastMonth extends MeasurementPeriod("last-month")

  val all = Set(LastTwentyFourHours, LastSevenDays, LastTwoWeeks, LastMonth)

  def fromString(str: String): Option[MeasurementPeriod] = all.find(_.repr == str)
}
