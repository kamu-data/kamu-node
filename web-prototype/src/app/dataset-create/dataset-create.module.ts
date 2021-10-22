import {ModuleWithProviders, NgModule} from "@angular/core";
import {MatMenuModule} from "@angular/material/menu";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {DatasetCreateComponent} from "./dataset-create.component";
import {SearchAdditionalButtonsModule} from "../components/search-additional-buttons/search-additional-buttons.module";
import {MatFormFieldModule} from "@angular/material/form-field";
import {NgbModule} from "@ng-bootstrap/ng-bootstrap";
import {FormsModule} from "@angular/forms";

@NgModule({
    imports: [
        MatMenuModule,
        MatIconModule,
        MatButtonModule,
        MatFormFieldModule,
        NgbModule,
        CommonModule,
        SearchAdditionalButtonsModule,
        FormsModule,
    ],
  exports: [DatasetCreateComponent],
  declarations: [DatasetCreateComponent]
})
export class DatasetCreateModule {
    // tslint:disable-next-line: no-any
    public static forRoot(): ModuleWithProviders<any> {
    return { ngModule: DatasetCreateModule };
  }
}