import { FormikProvider, useFormik } from "formik";
import { getStationDetails, updateStation } from "../api";
import { useParams } from "react-router-dom";
import * as yup from "yup";
import InputField from "../components/InputField";
import TextArea from "../components/TextArea";
import { useAuth } from "../AuthContext";
import { useQuery } from "react-query";
import { PrimaryButton } from "../components/PrimaryButton";
import { TertiaryButton } from "../components/TertiaryButton";
import { useNavigate } from "react-router-dom";
import { useQueryClient } from "react-query";

const attributeSchema = yup.object({
    name: yup.string().required("Name is required"),
    description: yup.string().required("Description is required"),
    location: yup.string().required("Location is required")
  });

interface AttributeUpdate extends yup.InferType<typeof attributeSchema> {}

export const PlantEdit = () => {
  const { plantId } = useParams();
  const auth = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { data } = useQuery([`plants/${plantId}/details`], () => getStationDetails(plantId ?? "")(auth.token ?? ""));

  const formAttributes = useFormik({
    enableReinitialize: true,
    initialValues: data?.station ?? { name: "", location: "", description: ""},
    validationSchema: attributeSchema,
    onSubmit: (values: AttributeUpdate) => {
        queryClient.invalidateQueries("plants");
        updateStation(plantId ?? "", values)(auth.token ?? "").then(() => navigate(`/plants/${plantId}`))
    }
  });

    return (
        <div className="mt-5 lg:mt-6 bg-white shadow sm:rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <FormikProvider value={formAttributes}>
            <form
              className="space-y-10 divide-y divide-gray-900/10"
              onSubmit={formAttributes.handleSubmit}
            >
              <div className="rid grid-cols-1 gap-x-8 gap-y-8 md:grid-cols-3">
                <div>
                  <div>
                    <h3 className="text-lg leading-6 font-medium text-gray-900">
                      Plant settings
                    </h3>
                    <p className="mt-1 max-w-2xl text-sm text-gray-500">
                      These are settings for your station
                    </p>
                  </div>

                  <InputField
                    type="text"
                    id="name"
                    name="name"
                    label="Name"
                    placeholder="My nice plant ..."
                    value={formAttributes.values.name}
                    onChange={formAttributes.handleChange}
                    helperText="Name is required"
                  />

                    <InputField
                    type="text"
                    id="location"
                    name="location"
                    label="Location"
                    placeholder="Living ..."
                    value={formAttributes.values.name}
                    onChange={formAttributes.handleChange}
                    helperText="Location is required"
                  />

                  <TextArea
                    id="description"
                    name="description"
                    label="Description"
                    type="description"
                    value={formAttributes.values.description}
                    onChange={formAttributes.handleChange}
                    helperText="Description is required"
                  />

                  <div className="pt-5">
                    <div className="flex justify-end">
                      <TertiaryButton text="Cancel" href={`/plants/${plantId}`} />
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
