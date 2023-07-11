package co.mycelium.db

import cats.tagless.{Derive, FunctorK}
import co.mycelium.domain._
import doobie._
import doobie.implicits._
import doobie.postgres.implicits._

import java.util.UUID

trait StationMeasurementRepository[F[_]] {
  def insertMany(stationId: UUID, measurements: List[StationMeasurement]): F[Int]
}

object StationMeasurementRepository {
  implicit val functorK: FunctorK[StationMeasurementRepository] = Derive.functorK
}

object DoobieStationMeasurementRepository extends StationMeasurementRepository[ConnectionIO] {
  override def insertMany(stationId: UUID, measurements: List[StationMeasurement]): ConnectionIO[Int] =
    Update[(UUID, StationMeasurement)]("insert into station_measurements (station_id, occurred_on, battery_voltage, temperature, humidity, lux, soil_pf, tank_pf) values (?, ?, ?, ?, ?, ?, ?, ?)")
      .updateMany(measurements.map(x => (stationId, x)))
}
