package co.mycelium

import zio._
import zio.stream.ZSink

object Main extends ZIOAppDefault {
  override def run: ZIO[ZIOAppArgs with Scope, Any, Any] = {

    val insert =
      PerenualClient.species.run(plantInsert)


    insert.repeat(Schedule.duration(1.day)).provide(
      DoobiePerenualRateLimitter.live(300),
      SttpPerenualClient.live,
      Transactors.layer,
      DoobiePlantDetailRepository.live,
      DbConfig.layer
    )

    ZIO.succeed(println("Hi")).repeat(Schedule.fixed(2.second))
  }

  val plantInsert: ZSink[PlantDetailRepository, Throwable, PlantDetail, Nothing, Unit] =
    ZSink.foreach(d => ZIO.serviceWithZIO[PlantDetailRepository](_.insert(d)))

}
