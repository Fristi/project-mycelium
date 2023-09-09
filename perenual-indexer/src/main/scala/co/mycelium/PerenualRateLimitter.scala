package co.mycelium

import zio.{Ref, Task, ZLayer}

import java.time.{Instant, LocalDate, LocalDateTime, ZoneId}

trait PerenualRateLimitter {
  def increaseRequestsToday(f: LatestIndexState => LatestIndexState): Task[Unit]

  def hasQuotaLeft: Task[Boolean]

  def getState: Task[LatestIndexState]
}



final case class InMemoryPerenualRateLimitter(state: Ref[LatestIndexState], limit: Int) extends PerenualRateLimitter {
  override def increaseRequestsToday(f: LatestIndexState => LatestIndexState): Task[Unit] =
    state.update(f)

  override def hasQuotaLeft: Task[Boolean] = state.get.map(_.requests < limit)

  override def getState: Task[LatestIndexState] = state.get
}

object InMemoryPerenualRateLimitter {
  def live(limit: Int) = ZLayer {
    for {
      state <- Ref.make(LatestIndexState(1, None, None, 0))
    } yield InMemoryPerenualRateLimitter(state, limit)
  }
}

final case class LatestIndexState(page: Int, id: Option[Int], lastUpdate: Option[Instant], requests: Int) {

  private def getRequestsToday = if(lastUpdate.exists(isSameDay)) requests else 0

  private def isSameDay(instant: Instant) =
    LocalDate.ofInstant(instant, ZoneId.of("Europe/Amsterdam")) isEqual LocalDate.now

  def withNextPage: LatestIndexState = copy(page = page + 1, requests = getRequestsToday + 1)
  def withLatestId(id: Int): LatestIndexState = copy(id = Some(id), requests = getRequestsToday + 1)
}

object LatestIndexState {
  val zero: LatestIndexState = LatestIndexState(1, None, None, 0)
}