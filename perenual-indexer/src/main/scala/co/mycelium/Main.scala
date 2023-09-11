package co.mycelium

import zio._
import zio.stream.ZSink

object Main extends ZIOAppDefault {
  override def run: ZIO[ZIOAppArgs with Scope, Any, Any] = {

    val insert =
      PerenualClient.species.run(plantInsert)

    insert.catchAllCause(err => ZIO.logErrorCause(err)).repeat(Schedule.fixed(1.day)).provide(
      DoobiePerenualRateLimitter.live(300),
      PerenualConfig.live,
      SttpPerenualClient.live,
      Transactors.live,
      DoobiePlantDetailRepository.live,
      DbConfig.live
    )
  }

  val plantInsert: ZSink[PlantDetailRepository, Throwable, PlantDetail, Nothing, Unit] =
    ZSink.foreach(d => ZIO.serviceWithZIO[PlantDetailRepository](_.insert(d)))

}
