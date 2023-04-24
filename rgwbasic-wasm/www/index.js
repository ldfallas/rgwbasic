import * as wasm from "rgwbasic-wasm";

window.appendElementLd = function(uline) {
/*    var e = document.getElementById("root");
    var newDiv = document.createElement('div');
    newDiv.innerHTML = value;
    e.appendChild(newDiv);*/
    window.consoleState.print(uline);
}

window.appendElementLn = function(uline) {
    window.consoleState.println(uline);
}

window.log = function(str) {
    console.log(str);
}

window.readLine = function(continueFunc) {
    let txt = prompt();
    setTimeout(() => continueFunc(txt));
}

var interpreter = wasm.GwInterpreterWrapper.new(); //wasm.GwWsmInterpreter.new();
//interpreter.start_interpreter();

window.interpreter = interpreter;

setTimeout( () => {
    var state = { lineTxt:'',
                  currentLine: 0,
                  maxLine: 25,
                  lineReadResultPromise: null
                };

    //    window.addEventListener('keydown', (e) => {
    // window.addEventListener('keydown', (e) => {
    //     console.log(e.target.tagName + " 1");
    //     if (e.keyCode == 32 && e.target == document.body) {
    //         console.log('dd');
    //         e.preventDefault();
    //         e.stopPropagation();
    //     }
    //     console.log(e.target.tagName + " 2");
    // });
    window.consoleState = state;
    state.print = (str) => {
        var e = document.getElementsByClassName('focused')[0];
        e.innerHTML = e.innerHTML + str;
        state.lineTxt = state.lineTxt + str;
    };
    state.println = (str) => {
        var e = document.getElementsByClassName('focused')[0];
        e.innerHTML = e.innerHTML + str;
        state.lineTxt = state.lineTxt + str;
        performEnter(state);
    };
    state.readLine = () =>{
        state.lineReadResultPromise = new Promise((resolve) => {
            state.lineReadResultPromiseResolve = resolve;
        });
        return state.lineReadResultPromise;
    };

    const step = (interpreter) => {
        interpreter.step_current_program();
        requestAnimationFrame(() => step(interpreter))
    }

    const replLoop =  () => {
        state.readLine().then( (line) => {
            if (line.toLowerCase() == 'run') {
                console.log("catched 'run' invocation");
                //interpreter.start_step_execution();
                //requestAnimationFrame(() => step(interpreter))
                interpreter.run_evaluator_loop(interpreter);
            } else {
                interpreter.eval_in_interpreter(line);
            }

            requestAnimationFrame(replLoop);
        });
    };
    requestAnimationFrame(replLoop);
    
    state.keyUpHandler = (e) => {
        if (!state.lineReadResultPromiseResolve) {
            return;
        }
        if (e.keyCode === 13) {
            var oldStr = state.lineTxt;
            performEnter(state);
            //interpreter.eval_in_interpreter(oldStr);
            state.lineReadResultPromiseResolve(oldStr);
            state.lineReadResultPromiseResolve = null;
            state.lineReadResultPromise = null;

        } else if (e.keyCode === 8) { // << this only works with keydown
            var subs = e.target.innerHTML;
            e.target.innerHTML = subs.substr(0, subs.length - 1);
            state.lineTxt = subs.substr(0, subs.length - 1);
        } else if (e.key.length === 1)  {
            e.target.innerHTML = e.target.innerHTML + e.key/*String.fromCharCode(e.keyCode)*/;
            state.lineTxt = state.lineTxt + e.key/*String.fromCharCode(e.keyCode)*/;
        }
        if (e.key == ' ') {
            // Avoid auto scrolling
            e.preventDefault();
        }
    };
    var restoreStateToCurrentLine = function(state, mainElement) {
        var rowPre = state.currentLinePre;
        rowPre.removeEventListener('keyup', state.keyUpHandler);
        rowPre.classList.remove('focused');
        rowPre.removeAttribute('tabIndex');
    };
    var performEnter = function(state) {
        restoreStateToCurrentLine(state);
        if (state.maxLine === (state.currentLine + 1)) {
            var currentDiv = state.currentLinePre.parentElement;
            var main = currentDiv.parentElement;
            main.removeChild(main.children[0]);
            var rowDiv = document.createElement('div');
            var rowPre = document.createElement('pre');
            rowPre.classList.add('focused');
            rowPre.tabIndex = -1;
            state.currentLinePre = rowPre;
            rowPre.addEventListener('keyup', state.keyUpHandler);
            rowDiv.appendChild(rowPre);
            main.appendChild(rowDiv);
            rowPre.focus();

        } else {
            var nextDiv = state.currentLinePre.parentElement.nextSibling;
            if (nextDiv && nextDiv.childElementCount == 1) {
                var nextPre = nextDiv.children[0];
                state.currentLinePre = nextPre;
                state.currentLine++;
                nextPre.addEventListener('keyup', state.keyUpHandler);
                nextPre.classList.add('focused');
                nextPre.setAttribute('tabIndex', '-1');
                nextPre.focus();
            }
        }
        state.lineTxt = '';
    };
    var main = document.getElementById('main');
    for(var i = 0; i < 25;i++) {
        var rowDiv = document.createElement('div');
        let rowPre = document.createElement('pre');
        if (i === 0) {
            rowPre.classList.add('focused');
            rowPre.tabIndex = -1;
            state.currentLinePre = rowPre;
            rowPre.addEventListener('keyup', state.keyUpHandler);
            setTimeout(() => {
                rowPre.focus();
            });
//            rowPre.addEventListener('keydown', (e) => {e.preventDefault()} );
        }
        rowDiv.appendChild(rowPre);
        main.appendChild(rowDiv);
    }
    console.log('done');
});      

