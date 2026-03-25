
type InfoBarParams = {
  github: string;
  docs: string;
  themeMode: "light" | "dark";
  onToggleTheme: () => void;
};

export function InfoBar(params: InfoBarParams) {
  return (
    <div className="bg-ctp-crust py-4 px-6 flex justify-between items-center shrink-0">
      <div className="flex gap-2 items-end">
        <h1 className="font-bold text-2xl text-ctp-mauve uppercase">RustSynth</h1>
        <span className="text-xs text-ctp-subtext0 font-mono pb-1">v0.1.0</span>
      </div>
      <div className="flex justify-center items-center gap-8">
        <a
          href={params.github}
          target="_blank"
          rel="noreferrer"
          className="flex gap-2 font-bold justify-center items-center text-ctp-text hover:text-ctp-mauve transition-colors"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16">
            <path
              fill="currentColor"
              fillRule="evenodd"
              d="M7.976 0A7.977 7.977 0 0 0 0 7.976c0 3.523 2.3 6.507 5.431 7.584c.392.049.538-.196.538-.392v-1.37c-2.201.49-2.69-1.076-2.69-1.076c-.343-.93-.881-1.175-.881-1.175c-.734-.489.048-.489.048-.489c.783.049 1.224.832 1.224.832c.734 1.223 1.859.88 2.3.685c.048-.538.293-.88.489-1.076c-1.762-.196-3.621-.881-3.621-3.964c0-.88.293-1.566.832-2.153c-.05-.147-.343-.978.098-2.055c0 0 .685-.195 2.201.832c.636-.196 1.322-.245 2.007-.245s1.37.098 2.006.245c1.517-1.027 2.202-.832 2.202-.832c.44 1.077.146 1.908.097 2.104a3.16 3.16 0 0 1 .832 2.153c0 3.083-1.86 3.719-3.62 3.915c.293.244.538.733.538 1.467v2.202c0 .196.146.44.538.392A7.98 7.98 0 0 0 16 7.976C15.951 3.572 12.38 0 7.976 0"
              clipRule="evenodd"
            />
          </svg>
          GITHUB
        </a>
        <a
          href={params.docs}
          target="_blank"
          rel="noreferrer"
          className="flex gap-1 font-bold justify-center items-center text-ctp-text hover:text-ctp-mauve transition-colors"
          id="tour-docs"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16">
            <path
              fill="currentColor"
              d="M3 3a2 2 0 0 1 2-2h3.586a1.5 1.5 0 0 1 1.06.44l2.915 2.914A1.5 1.5 0 0 1 13 5.414V13a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zm2-1a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V6H9.5A1.5 1.5 0 0 1 8 4.5V2zm4.5 3h2.293L9 2.207V4.5a.5.5 0 0 0 .5.5m-4 3a.5.5 0 0 0 0 1h5a.5.5 0 0 0 0-1zM5 10.5a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m.5 1.5a.5.5 0 0 0 0 1h5a.5.5 0 0 0 0-1z"
            />
          </svg>
          DOCS
        </a>
        <button
          type="button"
          onClick={params.onToggleTheme}
          className="flex gap-1 font-bold justify-center items-center text-ctp-text hover:text-ctp-mauve transition-colors cursor-pointer"
        >
          {params.theme === "dark" ? (
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M8.463 15.538Q7 14.075 7 12t1.463-3.537T12 7t3.538 1.463T17 12t-1.463 3.538T12 17t-3.537-1.463M5 13H1v-2h4zm18 0h-4v-2h4zM11 5V1h2v4zm0 18v-4h2v4zM6.4 7.75L3.875 5.325L5.3 3.85l2.4 2.5zm12.3 12.4l-2.425-2.525L17.6 16.25l2.525 2.425zM16.25 6.4l2.425-2.525L20.15 5.3l-2.5 2.4zM3.85 18.7l2.525-2.425L7.75 17.6l-2.425 2.525z"
            />
          </svg>) : (
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 21q-3.75 0-6.375-2.625T3 12t2.625-6.375T12 3q.35 0 .688.025t.662.075q-1.025.725-1.638 1.888T11.1 7.5q0 2.25 1.575 3.825T16.5 12.9q1.375 0 2.525-.613T20.9 10.65q.05.325.075.662T21 12q0 3.75-2.625 6.375T12 21"/></svg>)}
          {params.theme === "dark" ? "LIGHT" : "DARK"}
        </button>
      </div>
    </div>
  );
}