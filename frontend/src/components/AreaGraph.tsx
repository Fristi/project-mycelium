import { AreaChart, Area, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import moment from "moment";

type DataPoint = {
  on: string;
  value: number;
};

type Props = {
  header: string;
  label: string;
  data: DataPoint[];
};

function formatDate(tickItem: string) {
  return moment(tickItem).format("MMM Do YYYY HH:MM");
}

export default (props: Props) => {
  return (
    <div className="bg-white shadow sm:rounded-lg mb-5">
      <div className="px-4 py-5 sm:p-6">
        <h3 className="text-lg leading-6 font-medium text-gray-900 mb-3 ">{props.header}</h3>
        <div className="mt-2 text-sm text-gray-500">
          <ResponsiveContainer width="100%" height={200}>
            <AreaChart data={props.data}>
              <defs>
                <linearGradient id="gradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#15803d" stopOpacity={0.1} />
                  <stop offset="95%" stopColor="#FFFFFF" stopOpacity={0.1} />
                </linearGradient>
              </defs>
              <CartesianGrid vertical={false} stroke="#DDD" />
              <XAxis dataKey="on" tickFormatter={formatDate} />
              <YAxis />
              <Tooltip />
              <Line
                type="monotone"
                unit={props.label}
                strokeLinecap="round"
                strokeWidth={2}
                style={{ strokeDasharray: `40% 60%` }}
                dataKey="value"
                stroke="#15803d"
                dot={false}
                legendType="none"
              />
              <Area type="monotone" dataKey="value" stroke="#15803d" strokeWidth={2} fillOpacity={1} fill="url(#gradient)" />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
};
