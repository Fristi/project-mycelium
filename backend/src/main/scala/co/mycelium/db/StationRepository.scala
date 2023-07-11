package co.mycelium.db

import cats.tagless.{Derive, FunctorK}
import cats.implicits._
import co.mycelium.domain._
import doobie._
import doobie.implicits._
import doobie.postgres.implicits._

import java.time.Instant
import java.util.UUID

trait StationRepository[F[_]] {
  def insert(station: Station, on: Instant): F[Int]
  def listByUserId(userId: String): F[List[Station]]
  def findById(id: UUID, userId: String): F[Option[Station]]
  def delete(id: UUID, userId: String): F[Int]
}

object StationRepository {
  implicit val functorK: FunctorK[StationRepository] = Derive.functorK
}

object DoobieStationRepository extends StationRepository[ConnectionIO] {
  def insert(station: Station, on: Instant): ConnectionIO[Int] = {

    def insertIntoStations =
      sql"INSERT INTO stations (id, mac_addr, name, location, description, user_id, watering_schedule, created) VALUES (${station.id}, ${station.mac}, ${station.name}, ${station.location}, ${station.description}, ${station.userId}, ${station.wateringSchedule}, $on)".update.run

    insertIntoStations <* DoobieStationLogRepository.insert(StationLog(station.id, on, StationEvent.ScheduleChanged(station.wateringSchedule)))
  }

  def listByUserId(userId: String): ConnectionIO[List[Station]] =
    sql"SELECT id, mac_addr, name, location, description, watering_schedule, user_id, created, updated FROM stations where user_id = $userId".query[Station].to[List]

  def findById(id: UUID, userId: String): ConnectionIO[Option[Station]] =
    sql"SELECT id, mac_addr, name, location, description, watering_schedule, user_id, created, updated FROM stations WHERE id = $id AND user_id = $userId".query[Station].option

  def delete(id: UUID, userId: String): ConnectionIO[Int] =
    sql"DELETE FROM stations WHERE id = $id AND user_id = $userId".update.run

}
