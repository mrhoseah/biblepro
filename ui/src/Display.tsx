export default function Display() {
  return (
    <div id="display-root" className="h-screen w-screen overflow-hidden bg-black">
      <img
        id="bp-slide"
        src=""
        alt=""
        className="h-full w-full object-contain"
        draggable={false}
      />
    </div>
  );
}
