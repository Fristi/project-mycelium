import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
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
        <h3 className="text-lg leading-6 font-medium text-gray-900 mb-3 ">
          {props.header}
        </h3>
        <div className="mt-2 text-sm text-gray-500">
          <ResponsiveContainer width="100%" height={200}>
            <AreaChart data={props.data}>
              <defs>
                <linearGradient id="gradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#15803d" stopOpacity={0.2} />
                  <stop offset="95%" stopColor="#052e16" stopOpacity={0.8} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="1 3" />
              <XAxis dataKey="on" tickFormatter={formatDate} />
              <YAxis />
              <Tooltip />
              <Area
                strokeOpacity={0.4}
                strokeWidth={1}
                isAnimationActive={false}
                fill="url(#gradient)"
                type="monotone"
                name={props.label}
                dataKey="value"
                activeDot={{ r: 4 }}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
};
