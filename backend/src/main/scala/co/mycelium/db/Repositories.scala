package co.mycelium.db

import cats.effect.kernel.MonadCancelThrow
import cats.tagless.{Derive, FunctorK}
import doobie._

trait Repositories[F[_]] {
  def stationLog: StationLogRepository[F]
  def stations: StationRepository[F]
  def measurements: StationMeasurementRepository[F]
}

object DoobieRepositories extends Repositories[ConnectionIO] {
  override def stationLog: StationLogRepository[ConnectionIO] = DoobieStationLogRepository
  override def stations: StationRepository[ConnectionIO] = DoobieStationRepository
  override def measurements: StationMeasurementRepository[ConnectionIO] = DoobieStationMeasurementRepository
}

object Repositories {
  implicit val functorK: FunctorK[Repositories] = Derive.functorK

  def fromTransactor[F[_]: MonadCancelThrow](transactor: Transactor[F]): Repositories[F] =
    FunctorK[Repositories].mapK(DoobieRepositories)(transactor.trans)
}
