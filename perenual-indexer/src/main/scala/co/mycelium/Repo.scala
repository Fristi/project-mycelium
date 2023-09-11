package co.mycelium

import doobie._
import doobie.implicits._
import doobie.postgres.implicits._
import doobie.util.transactor.Transactor
import zio.{Task, ZLayer}
import zio.interop.catz._

trait PlantDetailRepository {
  def insert(plantDetail: PlantDetail): Task[Int]
}

final case class DoobiePlantDetailRepository(transactor: Transactor[Task]) extends PlantDetailRepository  {

  def insert(plantDetail: PlantDetail): Task[Int] =
    sql"INSERT INTO plant_details (id, common_name, scientific_name, other_name, family, origin, type, dimension, cycle, propagation, watering, watering_period, sunlight, pruning_month, seeds, maintenance, soil, growth_rate, drought_tolerant, salt_tolerant, thorny, invasive, tropical, indoor, care_level, pest_susceptibility, pest_susceptibility_api, flowers, flowering_season, flower_color, cones, fruits, edible_fruit, edible_fruit_taste_profile, fruit_nutritional_value, fruit_color, harvest_season, leaf, leaf_color, edible_leaf, cuisine, medicinal, poisonous_to_humans, poisonous_to_pets, description) VALUES (${plantDetail.id}, ${plantDetail.common_name}, ${plantDetail.scientific_name}, ${plantDetail.other_name}, ${plantDetail.family}, ${plantDetail.origin}, ${plantDetail.`type`}, ${plantDetail.dimension}, ${plantDetail.cycle}, ${plantDetail.propagation}, ${plantDetail.watering}, ${plantDetail.watering_period}, ${plantDetail.sunlight}, ${plantDetail.pruning_month}, ${plantDetail.seeds}, ${plantDetail.maintenance}, ${plantDetail.soil}, ${plantDetail.growth_rate}, ${plantDetail.drought_tolerant}, ${plantDetail.salt_tolerant}, ${plantDetail.thorny}, ${plantDetail.invasive}, ${plantDetail.tropical}, ${plantDetail.indoor}, ${plantDetail.care_level}, ${plantDetail.pest_susceptibility}, ${plantDetail.pest_susceptibility_api}, ${plantDetail.flowers}, ${plantDetail.flowering_season}, ${plantDetail.flower_color}, ${plantDetail.cones}, ${plantDetail.fruits}, ${plantDetail.edible_fruit}, ${plantDetail.edible_fruit_taste_profile}, ${plantDetail.fruit_nutritional_value}, ${plantDetail.fruit_color}, ${plantDetail.harvest_season}, ${plantDetail.leaf}, ${plantDetail.leaf_color}, ${plantDetail.edible_leaf}, ${plantDetail.cuisine}, ${plantDetail.medicinal}, ${plantDetail.poisonous_to_humans}, ${plantDetail.poisonous_to_pets}, ${plantDetail.description}) ON CONFLICT DO NOTHING"
      .update
      .run
      .transact(transactor)

}

object DoobiePlantDetailRepository {
  val live: ZLayer[Transactor[Task], Nothing, PlantDetailRepository] =
    ZLayer.fromFunction(tx => DoobiePlantDetailRepository(tx))
}

object DoobiePernualRateLimitterQueries {
  def upsert(latestIndexState: LatestIndexState): ConnectionIO[Int] =
    sql"INSERT INTO pernual_state (last_seen_page, last_seen_id, requests, last_update) VALUES (${latestIndexState.page}, ${latestIndexState.id}, ${latestIndexState.requests}, now()) ON CONFLICT (id) DO UPDATE SET last_seen_page = EXCLUDED.last_seen_page, last_seen_id = EXCLUDED.last_seen_id, requests = EXCLUDED.requests, last_update = EXCLUDED.last_update".update.run

  def getState =
    sql"SELECT last_seen_page, last_seen_id, last_update, requests FROM pernual_state".query[LatestIndexState].option
}

final case class DoobiePerenualRateLimitter(transactor: Transactor[Task], limit: Int) extends PerenualRateLimitter {

  private def getState_ = DoobiePernualRateLimitterQueries.getState.map(_.getOrElse(LatestIndexState.zero))
  override def increaseRequestsToday(f: LatestIndexState => LatestIndexState): Task[Unit] =
    getState_.flatMap(state => DoobiePernualRateLimitterQueries.upsert(f(state))).transact(transactor).unit

  override def hasQuotaLeft: Task[Boolean] = getState.map(_.getRequestsToday < limit)

  override def getState: Task[LatestIndexState] = getState_.transact(transactor)
}

object DoobiePerenualRateLimitter {
  def live(limit: Int): ZLayer[Transactor[Task], Nothing, DoobiePerenualRateLimitter] =
    ZLayer.fromFunction(tx => DoobiePerenualRateLimitter(tx, limit))
}