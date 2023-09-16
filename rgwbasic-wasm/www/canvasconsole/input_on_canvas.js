import * as wasm from "rgwbasic-wasm";

//import * as wasm from "rgwbasic-wasm";

class ConsoleManager {
   constructor(canvasElement, container) {
      this.maxLine = 25;
      this.maxCol = 40;
      this.currentLine = 0;
      this.currentCol = 0;
      this.top = 0;
      this.left = 0;
      this.initializeCanvas(canvasElement, container);
   }
   initializeCanvas(canvasElement, container) {
      let ctx = canvasElement.getContext('2d');
      //ctx.font = "16pt Monospace";
      let fontsizeInPoints = 12;
      ctx.font = `${fontsizeInPoints}pt Monospace`;

      let textMetrics = ctx.measureText('0');
      //this.lineHeight = textMetrics.actualBoundingBoxAscent + textMetrics.actualBoundingBoxDescent;
      this.lineHeight = Math.trunc(fontsizeInPoints * 1.3);
      this.charWidth = textMetrics.actualBoundingBoxLeft + textMetrics.actualBoundingBoxRight;
      this.lineWidth = this.charWidth * this.maxCol;
      console.log(`lw: ${this.maxCol} ${this.lineWidth}`);
         
      this.ctx = ctx;
      this.backgroundColor = 'white';

      // Add a little bit of extra space
      //this.lineHeight += 4;
      if ((this.lineHeight * this.maxLine) > this.ctx.canvas.height) {
         alert('WARNING: canvas smaller than the max line , scrolling is not going to work');
      }

      let input = document.createElement('INPUT');
      input.style.font = ctx.font;
      container.appendChild(input);
      input.style.position = 'absolute';
      input.style.top = '0px';
      input.style.left = '0px';
      input.style.outline = 0;
      input.style.borderStyle = 'none';
      input.style.margin = 0;
      input.style.padding = 0;
      input.style.backgroundColor = 'rgba(0, 0, 0, 0)'
      input.spellcheck = false;
      input.focus();
      input.addEventListener('keyup', this.getEnterHandler(input,ctx));
      input.disabled = true;
      this.input = input;
      console.log(`sizes cw: ${this.charWidth} lw:${this.lineWidth} lh:${this.lineHeight}`);

   }
   print(text) {
      console.log(`Print: ${text}`);
      let done = false;
      while(!done) {
         if (text.length == 0) {
            return;
         } else if ((text.length + this.currentCol) < this.maxCol) {
            console.log(`__${this.left}, ${this.top} ${text}`);
            this.ctx.fillText(text, this.left, this.top + this.lineHeight);
            this.left += this.charWidth * text.length;
            this.currentCol += text.length;

            this.input.style.left = `${this.currentCol*this.charWidth}px`;
            return;
         } else if ((text.length + this.currentCol) == this.maxCol) {
            console.log(`CC1: ${this.maxCol}, ${this.currentCol} , '${text}'`)
            this.printline(text);
            return;
         } else {
            console.log(`CC1: ${this.maxCol}, ${this.currentCol}`)
            let currentLinePrint = text.substr(0, this.maxCol - this.currentCol);
            this.printline(currentLinePrint);
            text = text.substr(this.maxCol - this.currentCol);
            console.log("CC: "+this.currentCol + "." + text);
         }
      }
   }
    clear() {
        this.input.style.top = '0px';
        this.input.style.left = '0px';
        this.top = 0;
        this.left = 0;
        this.currentLine = 0;
        this.currentCol = 0;
        this.ctx.fillRect(0, 0, this.maxCol * charWidth, this.maxLine * lineHeight);
   }
   printline(text) {
      let ctx = this.ctx;
      let elementHeight = this.lineHeight;
      let horizonalOffset = this.charWidth * this.currentCol;
      console.log(`Horizontal offset ${horizonalOffset} '${text}'`);
      if ((this.currentLine + 1)  < this.maxLine) {
         this.top +=  elementHeight;
         this.input.style.top = `${this.top}px`;
         this.input.style.left = '0px';
         ctx.fillText(text,horizonalOffset,this.top);
      } else {
         this.input.style.left = '0px';
         ctx.fillText(text,horizonalOffset,this.top + elementHeight);
      }
      //element.value = '';
      this.incrementLine();
      this.left = 0;
   }

   getEnterHandler(element, ctx) {
      let elementHeight = this.lineHeight;
      return (ev) => {
         if (ev.key == 'Enter') {
            if (this.enterCallback) {
               this.enterCallback(element.value);
            }

             this.printline(element.value);
             element.value = '';
         }
      };
   }


   clear() {
      let color = this.ctx.fillStyle;
      this.ctx.fillStyle = this.backgroundColor;
      this.ctx.fillRect(0, 0, this.lineWidth,300);
      this.ctx.fillStyle = color;
      this.currentLine = 0;
      this.top = 0;
      this.left = 0;
      this.input.style.top = `${this.top}px`;
   }

   readTextLine() {
      this.input.disabled = false;
      this.input.focus();
      return new Promise((resolve) => {
          this.enterCallback = (value) => {
              resolve(value);
             this.enterCallback = null;
             this.input.disabled = true;
          }
      });
   }

   scrollOneRow() {
      const beforeLastRow = (this.maxLine - 1) * this.lineHeight
     
      let img = this.ctx.getImageData(0,
                                      this.lineHeight, 
                                      this.lineWidth,
                                      beforeLastRow + this.lineHeight);
      this.ctx.putImageData(img, 0,0);
      let color = this.ctx.fillStyle;
      this.ctx.fillStyle = this.backgroundColor;
      this.ctx.fillRect(0, beforeLastRow + this.lineHeight, this.lineWidth, this.lineHeight);
      this.ctx.fillStyle = color;
   }

   incrementLine() {
//      console.log(this.lineHeight);
//      this.ctx.strokeRect(this.currentLine*3, this.currentLine*this.lineHeight, 
//                          this.lineWidth, this.lineHeight );
      if (this.currentLine + 1 >= this.maxLine) {
         this.scrollOneRow();
      } else {
         this.currentLine++;
      }
      this.currentCol = 0;
   }
}


setTimeout( () => {
   let manager = new ConsoleManager(
      document.getElementById('theCanvas'),
      document.getElementById('container'));
   window.appendElementLd = function(uline) {
      manager.print(uline);
   }

   window.appendElementLn = function(uline) {
      manager.printline(uline);
   }

   window.clearconsole = () => { manager.clear(); }
   window.log = function(str) {
      console.log(str);
   }

   window.readLine = function(continueFunc) {
      manager.readTextLine().then(continueFunc);
   }

   let interpreter = wasm.GwInterpreterWrapper.new(); 

   window.interpreter = interpreter;
   const replLoop = () => {
        manager.printline("Ok");
        manager.readTextLine().then((line) => {
         if (line.toLowerCase() == 'run') {
             interpreter
                 .run_evaluator_loop(interpreter)
                 .then(replLoop);
         } else {
             interpreter.eval_in_interpreter(line);
             requestAnimationFrame(replLoop);
         }
      });
   };
   requestAnimationFrame(replLoop);
});
