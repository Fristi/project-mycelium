import Retrieve from "../Retrieve"
import { getStations } from "../api"
import PlantCard from "../components/PlantCard"
import { Station } from "../api"

export const PlantList = () => {

    const renderData = (stations: Station[]) => {
        return (
            <div className="mx-auto mt-16 grid max-w-2xl grid-cols-1 gap-x-8 gap-y-20 lg:mx-0 lg:max-w-none lg:grid-cols-3">
                {stations.map(s => <PlantCard key={s.id} station={s} />)}
            </div>
        )
    }

    return (<Retrieve 
        dataKey="stations"
        retriever={getStations()} 
        renderData={renderData} />)
}