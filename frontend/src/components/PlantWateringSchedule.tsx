import { WateringSchedule } from "../api"

type Props = {
    schedule: WateringSchedule
}

export default (props: Props) => {
    const schedule = props.schedule

    if(schedule._type == "Interval") {
        return (
            <span className="inline-flex items-center rounded-md bg-yellow-50 px-2 py-1 text-xs font-medium text-yellow-800 ring-1 ring-inset ring-yellow-600/20">
                Interval for {schedule.period}
            </span>
        )
    } else if (schedule._type == "Threshold") {
        return (
            <span className="nline-flex items-center rounded-md bg-indigo-50 px-2 py-1 text-xs font-medium text-indigo-700 ring-1 ring-inset ring-indigo-700/10">
                Threshold (&lt; {schedule.belowSoilPf} pF) for {schedule.period}
            </span>
        )
    }

    return (<></>)
}