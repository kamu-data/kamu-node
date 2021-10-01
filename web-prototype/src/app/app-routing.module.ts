import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {SearchComponent} from "./search/search.component";
import {LoginComponent} from "./auth/login/login.component";
import {DatasetComponent} from "./dataset-view/dataset.component";
import {DatasetCreateComponent} from "./dataset-create/dataset-create.component";
import {AccountComponent} from "./auth/account/account.component";

const routes: Routes = [
    {path: '', redirectTo: 'search', pathMatch: 'full'},
    {path: 'search', component: SearchComponent},
    {path: 'login', component: LoginComponent},
    {path: 'profile', component: AccountComponent},
    {
        path: 'dataset-view',
        component: DatasetComponent,
        children: [
            {path: ':id', component: DatasetComponent}
        ]
    },
    {
        path: 'dataset-create',
        component: DatasetCreateComponent,
        children: [
            {path: '', redirectTo: 'select-type', pathMatch: 'full'},
            {path: 'root', component: DatasetCreateComponent}
        ]
    }
];

@NgModule({
    imports: [RouterModule.forRoot(routes)],
    exports: [RouterModule]
})
export class AppRoutingModule { }
