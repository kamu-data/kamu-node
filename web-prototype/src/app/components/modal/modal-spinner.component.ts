import { Component } from '@angular/core';
import { DynamicComponent } from './dynamic.component';



@Component({
    selector:   'modal-spinner',

    template:   `
        <div class="modal__content">
            <div data-test-id="spinner" class="loader">Loading...</div>
        </div>
    `
})
export class ModalSpinnerComponent extends DynamicComponent {}
