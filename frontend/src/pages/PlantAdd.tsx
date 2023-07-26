import { FormikProvider, useFormik } from "formik";
import { toFormikValidationSchema } from "zod-formik-adapter";
import { AddPlantSchema } from "../schemas";
import * as z from "zod";
import { useQueryClient } from "react-query";
import { useNavigate, useParams } from "react-router-dom";
import InputField from "../components/InputField";
import { PrimaryButton } from "../components/PrimaryButton";
import TextArea from "../components/TextArea";
import { BleClient } from "@capacitor-community/bluetooth-le";
import { useEffect, useState } from "react";
import { CheckCircleIcon, ClockIcon, PlusCircleIcon, UserIcon, WifiIcon } from "@heroicons/react/24/outline";

type PlantAdd = z.infer<typeof AddPlantSchema>;

type OnboardingStateAwaitingSettings = { _type: "AwaitingSettings" };
type OnboardingStateProvisioningWifi = { _type: "ProvisioningWifi" };
type OnboardingStateComplete = { _type: "Complete" };
type OnboardingStateAwaitingAuthorization = { _type: "AwaitingAuthorization", url: string }
type OnboardingStateFailed = { _type: "Failed", error: string }

type OnboardingState = OnboardingStateAwaitingSettings | OnboardingStateProvisioningWifi | OnboardingStateComplete | OnboardingStateAwaitingAuthorization | OnboardingStateFailed;


const MYCELIUM_SERVICE = "00467768-6228-2272-4663-277478269000";
const MYCELIUM_STATE_SERVICE = "00467768-6228-2272-4663-277478269001";
const MYCELIUM_RPC_SERVICE = "00467768-6228-2272-4663-277478269002";

type OnboardingStateViewProps = {
  icon: React.ReactNode,
  header: string
  children: React.ReactNode
}

const OnboardingStateView: React.FC<OnboardingStateViewProps> = ({children, icon, header}) => {
  return (
    <div className="text-center">
      {icon}
      <h3 className="mt-2 text-sm font-semibold text-gray-900">{header}</h3>
      <p className="mt-1 text-sm text-gray-500">{children}</p>
    </div>
  )
}

export const PlantProvisioning = () => {
  const { deviceId } = useParams();

  if(deviceId == null) return (<p>Invalid device id</p>)

  const [state, setState] = useState<OnboardingState>({ _type: "AwaitingSettings" });

  const decodeState = (data: DataView) => {
    const decoder = new TextDecoder();
    const res: OnboardingState = JSON.parse(decoder.decode(data));
    return res;
  };


  useEffect(() => {
    const interval = setInterval(async () => {
      await BleClient.initialize();
      await BleClient.requestDevice({ services: [MYCELIUM_SERVICE] });
      await BleClient.connect(deviceId);
      const stateBytes = await BleClient.read(deviceId, MYCELIUM_SERVICE, MYCELIUM_STATE_SERVICE);
      await BleClient.disconnect(deviceId);
      setState(decodeState(stateBytes));
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  if(state._type == "AwaitingAuthorization") {
    return (
      <OnboardingStateView header="Awaiting authorization" icon={<UserIcon className="mx-auto h-12 w-12 text-gray-400"/>}>
        <p className="pb-2">To authorize this device, please click authorize and follow the steps</p>
        <PrimaryButton href={state.url} text="Authorize" />
      </OnboardingStateView>
    );
  } else if(state._type == "AwaitingSettings") {
    return (
      <OnboardingStateView header="Awaiting for settings" icon={<ClockIcon className="mx-auto h-12 w-12 text-gray-400"/>}>
        <p className="pb-2">Awaiting for settings to be entered</p>
      </OnboardingStateView>
    );
  } else if(state._type == "ProvisioningWifi") {
    return (
      <OnboardingStateView header="Connecting to WiFi" icon={<WifiIcon className="mx-auto h-12 w-12 text-gray-400"/>}>
        <p>The device is setting up a internet connection via the WiFi network</p>
      </OnboardingStateView>
    );
  } else if(state._type == "Failed") {
    return (
      <OnboardingStateView header="Internal error" icon={<></>}>
        <p className="pb-2">An error occurred: <i>{state.error}</i></p>
        <PrimaryButton href="/" text="Overview" />

      </OnboardingStateView>
    );
  } else {
    return (
      <OnboardingStateView header="Successfully added plant" icon={<CheckCircleIcon className="mx-auto h-12 w-12 text-gray-400"/>}>
        <p className="pb-2">Successfully added the plant, please return back to the overview</p>
        <PrimaryButton href="/" text="Overview" />

      </OnboardingStateView>
    );
  }
}

export const PlantAdd = () => {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const form = useFormik({
    enableReinitialize: true,
    initialValues: { name: "test", location: "test", description: "test", wifi_ssid: "Skynet", wifi_password: "Scheepsrecht*3" },
    validationSchema: toFormikValidationSchema(AddPlantSchema),
    onSubmit: (values: PlantAdd) => {
      queryClient.invalidateQueries("plants");

      const worker = async () => {
        await BleClient.initialize();
        const device = await BleClient.requestDevice({ services: [MYCELIUM_SERVICE] });
        await BleClient.connect(device.deviceId);
        const byteArray = new TextEncoder().encode(JSON.stringify(values));
        await BleClient.write(device.deviceId, MYCELIUM_SERVICE, MYCELIUM_RPC_SERVICE, new DataView(byteArray.buffer));        
        await BleClient.disconnect(device.deviceId);
        return device.deviceId;
      };

      worker()
        .catch(err => console.error(err))
        .then(id => navigate(`/plant-add/${id}`))
    },
  });


  return (
    <FormikProvider value={form}>
      <form className="space-y-10 divide-y divide-gray-900/10" onSubmit={form.handleSubmit}>
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
                  value={form.values.wifi_ssid}
                  onChange={form.handleChange}
                  helperText="SSID is required"
                />

                <InputField
                  type="password"
                  id="wifi_password"
                  name="wifi_password"
                  label="Password"
                  placeholder="Your password, like asterix"
                  value={form.values.wifi_password}
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
