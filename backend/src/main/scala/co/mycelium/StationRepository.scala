package co.mycelium

import doobie._
import doobie.implicits._
import doobie.postgres.implicits._

import java.util.UUID

final case class Station(
  id: UUID,
  mac: Array[Byte],
  name: String,
  location: String,
  description: String,
  userId: String
)

trait StationRepository[F[_]] {
  def insert(station: Station): F[Int]
  def listByUserId(userId: String): F[List[Station]]
  def findById(id: UUID): F[Option[Station]]
  def delete(id: UUID): F[Int]
}

object DoobieStationRepository extends StationRepository[ConnectionIO] {
  def insert(station: Station): ConnectionIO[Int] =
    sql"INSERT INTO stations VALUES (${station.id}, ${station.mac}, ${station.name}, ${station.location}, ${station.description}, ${station.userId})".update.run

  def listByUserId(userId: String): ConnectionIO[List[Station]] =
    sql"SELECT * FROM stations where user_id = $userId".query[Station].to[List]

  def findById(id: UUID): ConnectionIO[Option[Station]] =
    sql"SELECT * FROM stations where id = $id".query[Station].option

  def delete(id: UUID): ConnectionIO[Int] =
    sql"DELETE FROM stations WHERE id = $id".update.run
}
