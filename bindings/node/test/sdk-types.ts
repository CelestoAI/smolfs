import { SmolFS, doctor, type DoctorReport, type VolumeInfo } from "../index";

const fs = SmolFS.fromEnv();
const constructed = new SmolFS();
const report: DoctorReport = doctor();
const methodReport: DoctorReport = constructed.doctor();
const volume: VolumeInfo = fs.ensureVolume({ name: "demo", dev: true });

fs.init({
  name: "cloud",
  metadata: "redis://localhost:6379/1",
  storage: "s3",
  bucket: "https://example-bucket.s3.amazonaws.com"
});

fs.mount({
  name: volume.name,
  path: "./workspace",
  checkStorage: false
});

fs.flush(volume.name);
fs.unmount(volume.name, { force: true });
fs.status(volume.name);

report.storageBackend.found satisfies boolean;
methodReport.mountSupport.detail satisfies string;
