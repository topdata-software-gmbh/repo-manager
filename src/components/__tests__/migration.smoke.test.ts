import { DOMWrapper, flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import AddScanPathModal from "../AddScanPathModal.vue";
import ToastStack from "../ToastStack.vue";

type RepoToast = {
  id: number;
  kind: "success" | "error" | "info";
  title: string;
  message?: string;
};

const mocks = vi.hoisted(() => ({
  store: {
    scanModalOpen: true,
    toasts: [] as RepoToast[],
    addScanPath: vi.fn(),
    closeScanPathModal: vi.fn(),
    dismissToast: vi.fn(),
  },
}));

vi.mock("../../stores/repoStore", () => ({
  useRepoStore: () => mocks.store,
}));

function query<T extends Element>(selector: string): DOMWrapper<T> {
  const element = document.body.querySelector<T>(selector);
  if (!element) throw new Error(`missing element: ${selector}`);
  return new DOMWrapper(element);
}

beforeEach(() => {
  mocks.store.scanModalOpen = true;
  mocks.store.toasts.splice(0);
  mocks.store.addScanPath.mockReset().mockResolvedValue(true);
  mocks.store.closeScanPathModal.mockReset();
  mocks.store.dismissToast.mockReset();
  document.body.innerHTML = "";
});

describe("repo-manager design-system migration", () => {
  it("submits AddScanPathModal through the kit form and converts the depth", async () => {
    const wrapper = mount(AddScanPathModal, { attachTo: document.body });

    await query<HTMLInputElement>('input[placeholder="~/devel"]').setValue("~/devel");
    await query<HTMLInputElement>('input[type="number"]').setValue("5");
    await query<HTMLFormElement>("#scan-path-form").trigger("submit");
    await flushPromises();

    expect(mocks.store.addScanPath).toHaveBeenCalledTimes(1);
    expect(mocks.store.addScanPath).toHaveBeenCalledWith(
      expect.objectContaining({ path: "~/devel", maxDepth: 5, enabled: true }),
    );
    expect(mocks.store.closeScanPathModal).toHaveBeenCalledTimes(1);

    wrapper.unmount();
  });

  it("renders ToastStack through the kit and forwards dismissal", async () => {
    mocks.store.toasts.push({ id: 1, kind: "success", title: "Project folder added" });
    const wrapper = mount(ToastStack, { attachTo: document.body });

    expect(wrapper.text()).toContain("Project folder added");

    await wrapper.find(".td-toast__dismiss").trigger("click");

    expect(mocks.store.dismissToast).toHaveBeenCalledWith(1);
    wrapper.unmount();
  });
});
