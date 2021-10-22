import {ModuleWithProviders, NgModule} from "@angular/core";
import {MatMenuModule} from "@angular/material/menu";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {MatToolbarModule} from "@angular/material/toolbar";
import {MatFormFieldModule} from "@angular/material/form-field";
import {MatTableModule} from "@angular/material/table";
import {MatSidenavModule} from "@angular/material/sidenav";
import {NgbModule} from "@ng-bootstrap/ng-bootstrap";
import {FormsModule} from "@angular/forms";
import {DynamicTableComponent} from "./dynamic-table.component";

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
        FormsModule,
    ],
    exports: [DynamicTableComponent],
    declarations: [DynamicTableComponent]
})
export class DynamicTableModule {
    // tslint:disable-next-line: no-any
    public static forRoot(): ModuleWithProviders<any> {
        return {ngModule: DynamicTableModule};
    }
}