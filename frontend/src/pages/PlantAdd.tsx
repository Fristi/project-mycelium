import { FormikProvider, useFormik } from "formik";
import { toFormikValidationSchema } from "zod-formik-adapter";
import { AddPlantSchema } from "../schemas";
import * as z from "zod";
import { useQueryClient } from "react-query";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../AuthContext";
import InputField from "../components/InputField";
import { PrimaryButton } from "../components/PrimaryButton";
import TextArea from "../components/TextArea";
import { BleClient } from "@capacitor-community/bluetooth-le";
import Select from "../components/Select";
import { useEffect, useState } from "react";

type PlantAdd = z.infer<typeof AddPlantSchema>;

const MYCELIUM_SERVICE = "";

const useStationScan = () => {
  const [stations, setStations] = useState<string[]>([]);

  useEffect(() => {
    const scanDevices = async () => {
      await BleClient.initialize();
      await BleClient.requestLEScan(
        {
          services: [MYCELIUM_SERVICE],
        },
        (result) => {
          const stationSet = new Set(stations);
          stationSet.add(result.device.deviceId);
          setStations(Array.from(stationSet));
        },
      );
    };

    scanDevices().catch((err) => console.log(err));

    return () => {
      setTimeout(async () => {
        await BleClient.stopLEScan();
      }, 2000);
    };
  });

  return stations;
};

export const PlantAdd = () => {
  const auth = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const stations = useStationScan();
  const form = useFormik({
    enableReinitialize: true,
    initialValues: { name: "", location: "", description: "", wateringSchedule: { _type: "Threshold", period: "5 seconds", belowSoilPf: 400 }, wifi: { ssid: "", password: "" } },
    validationSchema: toFormikValidationSchema(AddPlantSchema),
    onSubmit: (values: PlantAdd) => {
      queryClient.invalidateQueries("plants");
    },
  });

  const handleStationChange = (ev: any) => {
    console.log(ev);
  };

  return (
    <FormikProvider value={form}>
      <form className="space-y-10 divide-y divide-gray-900/10" onSubmit={form.handleSubmit}>
        <div className="mt-5 lg:mt-6 bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <div className="rid grid-cols-1 gap-x-8 gap-y-8 md:grid-cols-3">
              <div>
                <div>
                  <h3 className="text-lg leading-6 font-medium text-gray-900">Station select</h3>
                  <p className="mt-1 max-w-2xl text-sm text-gray-500">Select the station</p>
                </div>

                <Select id="station" name="station" label="Station" onChange={handleStationChange} options={stations.map((s) => ({ label: s, value: s }))} />
              </div>
            </div>
          </div>
        </div>

        <div className="mt-5 lg:mt-6 bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <div className="rid grid-cols-1 gap-x-8 gap-y-8 md:grid-cols-3">
              <div>
                <div>
                  <h3 className="text-lg leading-6 font-medium text-gray-900">Plant basics</h3>
                  <p className="mt-1 max-w-2xl text-sm text-gray-500">These are basics for your plant</p>
                </div>

                <InputField
                  type="text"
                  id="name"
                  name="name"
                  label="Name"
                  placeholder="My nice plant ..."
                  value={form.values.name}
                  onChange={form.handleChange}
                  helperText="Name is required"
                />

                <InputField
                  type="text"
                  id="location"
                  name="location"
                  label="Location"
                  placeholder="Living ..."
                  value={form.values.name}
                  onChange={form.handleChange}
                  helperText="Location is required"
                />

                <TextArea
                  id="description"
                  name="description"
                  label="Description"
                  type="description"
                  value={form.values.description}
                  onChange={form.handleChange}
                  helperText="Description is required"
                />
              </div>
            </div>
          </div>
        </div>

        <div className="mt-5 lg:mt-6 bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <div className="rid grid-cols-1 gap-x-8 gap-y-8 md:grid-cols-3">
              <div>
                <div>
                  <h3 className="text-lg leading-6 font-medium text-gray-900">WiFi settings</h3>
                  <p className="mt-1 max-w-2xl text-sm text-gray-500">These are WiFi settings for your plant</p>
                </div>

                <InputField
                  type="text"
                  id="wifi_ssid"
                  name="wifi_ssid"
                  label="SSID"
                  placeholder="My SSID ..."
                  value={form.values.wifi.ssid}
                  onChange={form.handleChange}
                  helperText="SSID is required"
                />

                <InputField
                  type="password"
                  id="wifi_password"
                  name="wifi_password"
                  label="Password"
                  placeholder="Your password, like asterix"
                  value={form.values.wifi.password}
                  onChange={form.handleChange}
                  helperText="Password is required"
                />
              </div>
            </div>
          </div>
        </div>
        <div className="pt-5">
          <div className="flex justify-end">
            <PrimaryButton text="Add plant" />
          </div>
        </div>
      </form>
    </FormikProvider>
  );
};
