import {ModuleWithProviders, NgModule} from "@angular/core";
import {ModalService} from "./modal.service";
import {ModalComponent} from "./modal.component";
import {BlankComponent} from "./blank.component";
import {ModalDialogComponent} from "./modal-dialog.component";
import {CommonModule} from "@angular/common";
import {ModalImageComponent} from "./modal-image.component";
import {ModalSpinnerComponent} from "./modal-spinner.component";
import {ModalFilterComponent} from "./modal-filter.component";

@NgModule({
    imports: [CommonModule],
    declarations: [
        BlankComponent,
        ModalComponent,
        ModalDialogComponent,
        ModalImageComponent,
        ModalSpinnerComponent,
        ModalFilterComponent,
    ],
    entryComponents: [
        BlankComponent,
        ModalDialogComponent,
        ModalImageComponent,
        ModalSpinnerComponent,
        ModalFilterComponent,
    ],
    exports: [
        ModalComponent,
    ]
})
export class ModalModule {
    static forRoot(): ModuleWithProviders {
        return {
            ngModule: ModalModule,
            providers: [ModalService]
        };
    }
}
