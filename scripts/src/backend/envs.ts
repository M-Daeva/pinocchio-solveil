const schema = {
  QN_URL: (x: string) => x,
};

// infer the Env type from the schema
type Env = {
  [K in keyof typeof schema]: ReturnType<(typeof schema)[K]>;
};

// load and parse env
export const ENV = {} as Env;

for (const key in schema) {
  const raw = Bun.env[key];

  if (typeof raw === "undefined") {
    throw new Error(`Missing environment variable: ${key}`);
  }

  (ENV as any)[key] = (schema as any)[key](raw);
}
