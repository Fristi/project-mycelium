import { Station, WateringSchedule } from "../api";
import PlantLocation from "./PlantLocation";
import PlantWateringSchedule from "./PlantWateringSchedule";

type Props = {
  station: Station;
};

export default (props: Props) => {
  const station = props.station;
  const host = import.meta.env.MYCELIUM_HOST ?? "http://localhost:8080";

  return (
    <div className="flex flex-col items-start justify-between">
      <div className="relative w-full">
        <img
          src={`${host}/avatar/${station.id}`}
          alt={station.description}
          className="aspect-[16/9] w-full rounded-2xl bg-gray-100 object-cover sm:aspect-[2/1] lg:aspect-[3/2]"
        />
        <div className="absolute inset-0 rounded-2xl ring-1 ring-inset ring-gray-900/10" />
      </div>
      <div className="max-w-xl">
        <div className="group relative">
          <h3 className="mt-3 text-lg font-semibold leading-6 text-gray-900 group-hover:text-gray-600">
            <a href={`/plants/${station.id}`}>
              <span className="absolute inset-0" />
              {station.name}
            </a>
          </h3>
          <PlantLocation location={station.location} />
          <PlantWateringSchedule schedule={station.wateringSchedule} />
        </div>
      </div>
    </div>
  );
};
