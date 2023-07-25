import { useParams } from "react-router-dom";
import AreaGraph from "../components/AreaGraph";
import { StationDetails, StationMeasurement, getStationDetails } from "../api";
import Retrieve from "../Retrieve";
import PlantLocation from "../components/PlantLocation";
import PlantWateringSchedule from "../components/PlantWateringSchedule";

export const PlantView = () => {

    const { plantId } = useParams()

    const splitMeasurements = (measurements: StationMeasurement[]) => {
        return {
            batteryVoltage: measurements.map(x => ({ on: x.on, value: x.batteryVoltage})),
            humidity: measurements.map(x => ({ on: x.on, value: x.humidity})),
            lux: measurements.map(x => ({ on: x.on, value: x.lux})),
            soilPf: measurements.map(x => ({ on: x.on, value: x.soilPf})),
            tankPf: measurements.map(x => ({ on: x.on, value: x.tankPf})),
            temperature: measurements.map(x => ({ on: x.on, value: x.temperature}))
        };
    }

    

    const renderData = (stationDetails: StationDetails) => {
        const station = stationDetails.station
        const plantId = station.id
        const host = import.meta.env.MYCELIUM_HOST ?? "http://localhost:8080"
        const measurements = splitMeasurements(stationDetails.measurements)

        return (
            <>
            <div className="bg-white shadow">
        <div className="px-4 sm:px-6 lg:mx-auto lg:px-8">
          <div className="py-6 md:flex md:items-center md:justify-between lg:border-t lg:border-gray-200">
            <div className="flex-1 min-w-0">

              <div className="flex items-center">
                <img
                  className="hidden h-16 w-16 rounded-full sm:block"
                  src={`${host}/avatar/${plantId}`}
                  alt=""
                />
                <div>
                  <div className="flex items-center">
                    <img
                      className="h-16 w-16 rounded-full sm:hidden"
                      src={`${host}/avatar/${plantId}`}
                      alt=""
                    />
                    <div className="pl-7">
                    <h1 className="text-2xl font-bold leading-7 text-gray-900 sm:leading-9 sm:truncate">
                      {station.name}
                    </h1>
                    <p>
                      <PlantLocation location={station.location} />
                      <PlantWateringSchedule schedule={station.wateringSchedule} />
                    </p>
                    </div>
                    
                    
                  </div>

                </div>
              </div>
            </div>
          </div>
        </div>
      </div>


      <div className="mx-auto">

        <div className="mt-5 space-y-12 sm:grid sm:grid-cols-1 sm:gap-x-6 sm:gap-y-12 sm:space-y-0 lg:grid-cols-2 lg:gap-x-8">
          <AreaGraph header="Relative humidity" label="%" data={measurements.humidity} />
          <AreaGraph header="Temperature" label="Celsius" data={measurements.temperature} />
          <AreaGraph header="Soil capacitive" label="pF" data={measurements.soilPf} />
          <AreaGraph header="Watertank capacitive" label="pF" data={measurements.tankPf} />
          <AreaGraph header="Battery voltage" label="V" data={measurements.batteryVoltage} />
          <AreaGraph header="Lux" label="lx" data={measurements.lux} />
        </div>
      </div>
            </>
        )
    }

    return (
        <Retrieve 
            dataKey={`plant/${plantId}`} 
            retriever={getStationDetails(plantId ?? "")} renderData={renderData} />
    )
}