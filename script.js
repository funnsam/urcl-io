import init, * as urcl_io from "./pkg/urcl_io.js";
import { get_urcl } from "./urcl.js";

document.addEventListener("DOMContentLoaded", (_) => {
    hljs.registerLanguage("urcl", get_urcl);

    let input       = document.getElementById("input");
    let hl_box      = document.getElementById("hl_box");

    input.oninput  = (_) => { highlight() };
    input.onkeydown = (e) => {
        if (e.key == 'Tab') {
            e.preventDefault();
            let a = input.selectionStart+1;
            input.value = input.value.substring(0, input.selectionStart) + "\t" + input.value.substring(input.selectionEnd);
            input.setSelectionRange(a, a);
            highlight();
        };
    };
    input.onscroll = (_) => {
        hl_box.scrollTo(input.scrollLeft, input.scrollTop);
    };
    highlight();
    
    function highlight() {
        hl_box.innerHTML = hljs.highlight(input.value+"\n", {language: "urcl"}).value;
    }

	document.getElementById("test").onclick = (_) => {
		urcl_io.test(input.value);
	};
});

init().then(() => {
    urcl_io.init_panic_hook();
});
