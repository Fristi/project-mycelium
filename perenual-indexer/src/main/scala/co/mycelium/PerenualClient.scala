package co.mycelium

import sttp.client3._
import sttp.client3.httpclient.zio.HttpClientZioBackend
import sttp.client3.ziojson._
import sttp.model.StatusCode
import zio.json._
import zio.stream.ZStream
import zio.{Chunk, Task, ZIO, ZLayer}

trait PerenualClient {
  def getSpecies(page: Int): Task[Paged[PlantSummary]]

  def getPlantDetail(id: Int): Task[Option[PlantDetail]]

  def species: ZStream[Any, Throwable, PlantDetail]
}

object PerenualClient {
  def getSpecies(page: Int) = ZIO.environmentWithZIO[PerenualClient](_.get.getSpecies(page))

  def getPlant(id: Int) = ZIO.environmentWithZIO[PerenualClient](_.get.getPlantDetail(id))

  def species: ZStream[PerenualClient, Throwable, PlantDetail] = ZStream.environmentWithStream[PerenualClient](_.get.species)



}

final case class SttpPerenualClient(backend: SttpBackend[Task, Any], apiKey: String, rateLimitter: PerenualRateLimitter) extends PerenualClient {
  override def getSpecies(page: Int): Task[Paged[PlantSummary]] =
    basicRequest
      .get(uri"https://perenual.com/api/species-list?page=$page&key=$apiKey")
      .response(asJson[Paged[PlantSummary]])
      .send(backend)
      .flatMap(x => ZIO.fromEither(x.body))

  override def getPlantDetail(id: Int): Task[Option[PlantDetail]] =
    basicRequest
      .get(uri"https://perenual.com/api/species/details/$id?key=$apiKey")
      .response(asJson[PlantDetail])
      .send(backend)
      .flatMap(x => if (x.code == StatusCode.Ok) ZIO.fromEither(x.body).map(x => Some(x)) else ZIO.none)


  implicit val decoderPlantSummary: JsonDecoder[PlantSummary] = DeriveJsonDecoder.gen
  implicit val decoderPlantDetail: JsonDecoder[PlantDetail] = DeriveJsonDecoder.gen

  implicit def decoderPaged[T: JsonDecoder]: JsonDecoder[Paged[T]] = DeriveJsonDecoder.gen

  def species: ZStream[Any, Throwable, PlantDetail] = {
    def pages(state: LatestIndexState) = ZStream.unfoldZIO(state.page) { page =>
      getSpecies(page)
        .whenZIO(rateLimitter.hasQuotaLeft)
        .map {
          case Some(paged) =>
            val data = Chunk.from(paged.data)
            if (data.isEmpty) None else Some((data, page + 1))
          case None =>
            None
        }
    }
    .tap(_ => rateLimitter.increaseRequestsToday(_.withNextPage))

    def processSpecies(chunk: Chunk[PlantSummary]): ZStream[Any, Throwable, PlantDetail] =
      ZStream.fromChunk(chunk)
        .mapZIO(p => getPlantDetail(p.id).tap(_ => rateLimitter.increaseRequestsToday(_.withLatestId(p.id))).whenZIO(rateLimitter.hasQuotaLeft))
        .collectWhileSome
        .collectSome

    for {
      state <- ZStream.fromZIO(rateLimitter.getState)
      species <- pages(state)
      detail <- processSpecies(species)
    } yield detail
  }
}

object SttpPerenualClient {
  val live: ZLayer[PerenualRateLimitter, Throwable, SttpPerenualClient] = ZLayer.scoped {
    for {
      rateLimitter <- ZIO.service[PerenualRateLimitter]
      backend <- HttpClientZioBackend.scoped()
    } yield SttpPerenualClient(backend, "sk-jlA164f5fd04ad2632069", rateLimitter)
  }
}

case class PlantSummary(id: Int, common_name: String, scientific_name: Seq[String])

case class Paged[T](data: Seq[T])

case class PlantDetail(
  id: Int,
  common_name: String,
  scientific_name: Array[String],
  other_name: Array[String],
  family: Option[String],
  origin: Array[String],
  `type`: String,
  dimension: String,
  cycle: String,
  propagation: Array[String],
  watering: String,
  watering_period: Option[String],
  sunlight: Array[String],
  pruning_month: Array[String],
  seeds: Int,
  maintenance: Option[String],
  soil: Array[String],
  growth_rate: String,
  drought_tolerant: Boolean,
  salt_tolerant: Boolean,
  thorny: Boolean,
  invasive: Boolean,
  tropical: Boolean,
  indoor: Boolean,
  care_level: Option[String],
  pest_susceptibility: Array[String],
  pest_susceptibility_api: String,
  flowers: Boolean,
  flowering_season: Option[String],
  flower_color: String,
  cones: Boolean,
  fruits: Boolean,
  edible_fruit: Boolean,
  edible_fruit_taste_profile: String,
  fruit_nutritional_value: String,
  fruit_color: Array[String],
  harvest_season: Option[String],
  leaf: Boolean,
  leaf_color: Array[String],
  edible_leaf: Boolean,
  cuisine: Boolean,
  medicinal: Boolean,
  poisonous_to_humans: Int,
  poisonous_to_pets: Int,
  description: String
)


