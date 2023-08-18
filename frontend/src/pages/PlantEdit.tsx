import { FormikProvider, useFormik } from "formik";
import { getStationDetails, updateStation } from "../api";
import { useParams } from "react-router-dom";
import * as z from "zod";
import InputField from "../components/InputField";
import TextArea from "../components/TextArea";
import { useAuth } from "../AuthContext";
import { useQuery } from "react-query";
import { PrimaryButton } from "../components/PrimaryButton";
import { TertiaryButton } from "../components/TertiaryButton";
import { useNavigate } from "react-router-dom";
import { useQueryClient } from "react-query";
import { toFormikValidationSchema } from "zod-formik-adapter";
import { AttributeSchema } from "../schemas";

type AttributeUpdate = z.infer<typeof AttributeSchema>;

export const PlantEdit = () => {
  const { plantId } = useParams();
  const auth = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { data } = useQuery([`plants/${plantId}/details`], () => getStationDetails(plantId ?? "")(auth.token ?? ""));

  const BasicSettings = () => {
    const form = useFormik({
      enableReinitialize: true,
      initialValues: data?.station ?? { name: "", location: "", description: "" },
      validationSchema: toFormikValidationSchema(AttributeSchema),
      onSubmit: (values: AttributeUpdate) => {
        queryClient.invalidateQueries("plants");
        updateStation(plantId ?? "", values)(auth.token ?? "").then(() => navigate(`#/plants/${plantId}`));
      },
    });

    return (
      <div className="mt-5 lg:mt-6 bg-white shadow sm:rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <FormikProvider value={form}>
            <form className="space-y-10 divide-y divide-gray-900/10" onSubmit={form.handleSubmit}>
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

                  <div className="pt-5">
                    <div className="flex justify-end">
                      <TertiaryButton text="Cancel" href={`#/plants/${plantId}`} />
                      <PrimaryButton text="Save" />
                    </div>
                  </div>
                </div>
              </div>
            </form>
          </FormikProvider>
        </div>
      </div>
    );
  };

  return (
    <>
      <BasicSettings />
    </>
  );
};
