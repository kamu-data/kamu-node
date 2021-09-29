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
import {NgbModule} from "@ng-bootstrap/ng-bootstrap";
import {FormsModule} from "@angular/forms";
import {SearchSidenavComponent} from "../search/search-sidenav/search-sidenav.component";

@NgModule({
    imports: [
        MatMenuModule,
        MatIconModule,
        MatButtonModule,
        MatToolbarModule,
        MatFormFieldModule,
        MatTableModule,
        MatSidenavModule,
        NgbModule,
        CommonModule,
        SearchAdditionalButtonsModule,
        FormsModule,
    ],
  exports: [DatasetComponent],
  declarations: [DatasetComponent]
})
export class DatasetModule {
    public static forRoot(): ModuleWithProviders<any> {
    return { ngModule: DatasetModule };
  }
}