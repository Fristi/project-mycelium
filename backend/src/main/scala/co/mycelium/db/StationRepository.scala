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
  def findById(id: UUID): F[Option[Station]]
  def delete(id: UUID): F[Int]
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
    sql"SELECT * FROM stations where user_id = $userId".query[Station].to[List]

  def findById(id: UUID): ConnectionIO[Option[Station]] =
    sql"SELECT * FROM stations where id = $id".query[Station].option

  def delete(id: UUID): ConnectionIO[Int] =
    sql"DELETE FROM stations WHERE id = $id".update.run

}
