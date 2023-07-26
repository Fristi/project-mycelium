package co.mycelium

import cats._
import cats.data._
import cats.implicits._
import doobie._
import doobie.implicits._
import doobie.postgres.implicits._
import io.circe.Json
import doobie.postgres.circe.json.implicits._
import co.mycelium.CirceCodecs._
import co.mycelium.domain._
import cron4s.{Cron, CronExpr}
import io.circe.syntax._

package object db {

  implicit val putWateringSchedule: Put[WateringSchedule] = Put[Json].contramap(_.asJson)
  implicit val getWateringSchedule: Get[WateringSchedule] = Get[Json].temap(_.as[WateringSchedule].leftMap(_.message))

  implicit val putStationEvent: Put[StationEvent] = Put[Json].contramap(_.asJson)
  implicit val getStationEvent: Get[StationEvent] = Get[Json].temap(_.as[StationEvent].leftMap(_.message))

  implicit val putCron: Put[CronExpr] = Put[String].contramap(_.toString)
  implicit val getCron: Get[CronExpr] = Get[String].temap(x => Cron.parse(x).leftMap(_.getMessage))
}
