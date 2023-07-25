import axios from 'axios';

export type WateringScheduleInterval = {
    _type: "Interval",
    schedule: string,
    period: string
}

export type WateringScheduleThreshold = {
    _type: "Threshold",
    belowSoilPf: number,
    period: string
}

export type WateringSchedule = WateringScheduleInterval | WateringScheduleThreshold

export type Station = {
    id: string,
    name: string,
    description: string,
    location: string,
    wateringSchedule: WateringSchedule
}

export type StationMeasurement = {
    on: string,
    batteryVoltage: number,
    temperature: number,
    humidity: number,
    lux: number,
    soilPf: number,
    tankPf: number
}


export type StationDetails = {
    station: Station,
    measurements: StationMeasurement[]
}

const host = import.meta.env.MYCELIUM_HOST ?? "http://localhost:8080"

export function getStations(): (token: string) => Promise<[Station]> {
    return (token) => axios
        .get(`${host}/api/stations`, { headers: { "Authorization" : `Bearer ${token}`}} )
        .then(x => x.data);    
}

export function getStationDetails(id: string): (token: string) => Promise<StationDetails> {
    return (token) => axios
        .get(`${host}/api/stations/${id}`, { headers: { "Authorization" : `Bearer ${token}`}} )
        .then(x => x.data);  
}

