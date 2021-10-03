import {ModuleWithProviders, NgModule} from "@angular/core";
import {MatMenuModule} from "@angular/material/menu";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {DatasetComponent} from "./dataset.component";
import {SearchAdditionalButtonsModule} from "../components/search-additional-buttons/search-additional-buttons.module";
import {MatToolbarModule} from "@angular/material/toolbar";
import {MatFormFieldModule} from "@angular/material/form-field";
import {MatTableModule} from "@angular/material/table";
import {MatSidenavModule} from "@angular/material/sidenav";
import {NgbModule, NgbNavModule} from "@ng-bootstrap/ng-bootstrap";
import {FormsModule} from "@angular/forms";
import {DynamicTableModule} from "../components/dynamic-table/dynamic-table.module";
import {SearchSidenavModule} from "../search-sidenav/search-sidenav.module";
import {MatButtonToggleModule} from "@angular/material/button-toggle";

@NgModule({
    imports: [
        CommonModule,
        FormsModule,
        NgbModule,
        FormsModule,
        NgbNavModule,
        MatButtonToggleModule,
        DynamicTableModule,
        SearchSidenavModule,
        SearchAdditionalButtonsModule,
    ],
  exports: [DatasetComponent],
  declarations: [DatasetComponent]
})
export class DatasetModule {
    public static forRoot(): ModuleWithProviders<any> {
    return { ngModule: DatasetModule };
  }
}